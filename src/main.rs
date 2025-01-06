mod cli;
mod input;
mod svg;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Result};
use resvg::tiny_skia::{Color, Pixmap};
use usvg::{Transform, Tree};

fn main() -> Result<()> {
    let args = cli::Args::parse()?;

    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let data = input::parse(&args.input)?;
    if data.is_empty() {
        bail!("No data points found");
    }

    // read the first frame to get the size of the pixmap
    let mut pixmap = {
        let first_point = svg::render_svg(&data[0], &args);
        let first_tree = Tree::from_data(first_point.as_bytes(), &opt)?;

        let pixmap_size = first_tree
            .size()
            .to_int_size()
            .scale_by(args.scale)
            .ok_or_else(|| anyhow!("Failed to scale pixmap size"))?;

        Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or_else(|| anyhow!("Failed to create pixmap"))?
    };

    // prepare the output directory
    let output_dir = Path::new("frames");
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir)?;
    }
    std::fs::create_dir(output_dir)?;

    let concat_file_path = output_dir.join("instructions.txt");
    let mut concat_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&concat_file_path)?;

    for (i, point) in data.iter().enumerate() {
        let svg_data = svg::render_svg(&point, &args);
        let tree = Tree::from_data(&svg_data.as_bytes(), &opt)?;

        // Render the SVG to the Pixmap
        pixmap.fill(Color::TRANSPARENT);
        resvg::render(
            &tree,
            Transform::from_scale(args.scale, args.scale),
            &mut pixmap.as_mut(),
        );

        // must be relative to the concat file
        let output_file = format!("frame_{:05}.png", i);
        pixmap.save_png(output_dir.join(&output_file))?;
        writeln!(concat_file, "file '{}'", output_file)?;
        writeln!(
            concat_file,
            "duration {}",
            f32::min(point.duration, args.max_gap_seconds)
        )?;

        // due to a quirk in ffmpeg, we must specify the last frame twice
        if i == data.len() - 1 {
            writeln!(concat_file, "file '{}'", output_file)?;
        }
    }

    let frame_rate_arg = match args.rate {
        Some(rate) => &["-r", &rate.to_string()],
        None => &["-fps_mode", "vfr"],
    };

    // Spawn ffmpeg ourselves
    let status = Command::new("ffmpeg")
        .arg("-y")
        .args(&["-f", "concat"])
        .arg("-i")
        .arg(concat_file_path)
        .args(frame_rate_arg)
        .args(&["-pix_fmt", "argb"])
        .args(&["-c:v", "qtrle"])
        .arg(&args.output)
        .status()?;

    if !status.success() {
        bail!("ffmpeg failed with status: {}", status);
    }

    Ok(())
}
