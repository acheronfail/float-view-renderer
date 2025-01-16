mod cli;
mod err;
mod input;
mod render;

use std::io::Write;
use std::process::{Command, Stdio};

use input::DataPoint;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::err::Result;
use crate::render::*;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 960;

pub struct Context<'a> {
    args: &'a cli::Args,
    canvas: &'a mut Canvas<Window>,
    tex_creator: &'a TextureCreator<WindowContext>,
    font_title: &'a Font<'a, 'a>,
    font_small: &'a Font<'a, 'a>,
    font_regular: &'a Font<'a, 'a>,
}

fn render_frame(ctx: &mut Context, point: &DataPoint) -> Result<()> {
    ctx.canvas.set_draw_color(Color::RGBA(
        0,
        0,
        0,
        if ctx.args.transparent_bg { 0 } else { 255 },
    ));
    ctx.canvas.clear();

    let mut y = 20;

    y += Speedo {
        title: "Speed".to_string(),
        value: format!("{:.2} km/h", point.speed),
        min: 0.0,
        max: 60.0,
        color: Color::RED,
        ..Default::default()
    }
    .render(ctx, point.speed as f64, y as f64)?
    .1;

    y += Speedo {
        title: "Duty Cycle".to_string(),
        value: format!("{}%", point.duty_cycle),
        color: Color::MAGENTA,
        ..Default::default()
    }
    .render(ctx, point.duty_cycle as f64, y as f64)?
    .1;

    y += List::new(
        "Motor",
        vec![
            LabelValue::new("Current", &format!("{:.2} A", point.motor_current)),
            LabelValue::new(
                "Field Weakening",
                &format!("{:.2} A", point.field_weakening.unwrap_or(f32::NAN)),
            ),
        ],
    )
    .with_color(Color::CYAN)
    .render(ctx, y as f64)?
    .1;
    y += List::new(
        "Temps",
        vec![
            LabelValue::new("Motor", &format!("{:.2} °C", point.temp_motor)),
            LabelValue::new("Controller", &format!("{:.2} °C", point.temp_mosfet)),
        ],
    )
    .with_color(Color::RGB(255, 165, 0))
    .render(ctx, y as f64)?
    .1;

    List::new(
        "Power",
        vec![
            LabelValue::new(
                "Voltage (per cell)",
                &format!("{:.2} V", point.batt_voltage / ctx.args.cell_count as f32),
            ),
            LabelValue::new("Voltage", &format!("{:.2} V", point.batt_voltage)),
            LabelValue::new("Current", &format!("{:.2} A", point.batt_current)),
            LabelValue::new(
                "Wattage",
                &format!(
                    "{} W",
                    (point.batt_voltage * point.batt_current).round() as usize
                ),
            ),
        ],
    )
    .with_color(Color::YELLOW)
    .render(ctx, y as f64)?
    .1;

    Ok(())
}

fn main() -> Result<()> {
    let args = cli::Args::parse()?;

    let data = input::parse(&args.input)?;
    if data.is_empty() {
        bail!("No data points found in input {}", args.input);
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init()?;

    let window = video_subsystem
        .window("SDL2 Video Capture", WIDTH, HEIGHT)
        .position_centered()
        .build()?;

    let pixel_format = if args.transparent_bg {
        PixelFormatEnum::ARGB32
    } else {
        PixelFormatEnum::IYUV
    };

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_target(pixel_format, WIDTH, HEIGHT)?;

    let font_title = ttf_context.load_font(&args.title_font, 20)?;
    let font_small = ttf_context.load_font(&args.font, 18)?;
    let font_regular = ttf_context.load_font(&args.font, 24)?;

    // Start ffmpeg process
    let dimensions = format!("{}x{}", WIDTH, HEIGHT);
    let mut ffmpeg = Command::new("ffmpeg")
        // overwrite
        .arg("-y")
        // input format
        .args(&["-f", "rawvideo"])
        // pixel format
        .args(&[
            "-pixel_format",
            if args.transparent_bg {
                "argb"
            } else {
                "yuv420p"
            },
        ])
        // video size
        .args(&["-video_size", dimensions.as_str()])
        // frame rate
        .args(&["-framerate", args.rate.to_string().as_str()])
        // input file
        .args(&["-i", "-"])
        // codec
        .args(if args.transparent_bg {
            vec!["-c:v", "qtrle"]
        } else {
            vec!["-c:v", "libx264", "-preset", "fast", "-crf", "23"]
        })
        // output format
        .args(&["-f", if args.transparent_bg { "mov" } else { "mp4" }])
        .args(if args.scale != 1.0 {
            vec![
                String::from("-vf"),
                format!(
                    "scale={scale:.2}*iw:{scale:.2}*ih:flags=lanczos",
                    scale = args.scale
                ),
            ]
        } else {
            vec![]
        })
        // output file
        .arg(&args.output)
        .stdin(Stdio::piped())
        .spawn()?;

    let ffmpeg_stdin = ffmpeg.stdin.as_mut().ok_or("Failed to open ffmpeg stdin")?;

    for point in data.iter() {
        let duration = point.duration.min(args.max_gap_seconds);
        let num_frames = (duration * args.rate).round() as usize;

        canvas.with_texture_canvas(&mut texture, |texture_canvas| {
            let mut ctx = Context {
                args: &args,
                canvas: texture_canvas,
                tex_creator: &texture_creator,
                font_small: &font_small,
                font_title: &font_title,
                font_regular: &font_regular,
            };

            if let Err(e) = render_frame(&mut ctx, &point) {
                eprintln!("Error rendering frame {}: {}", point.index, e);
            }
        })?;

        canvas.copy(&texture, None, None)?;

        let pixel_data = &canvas.read_pixels(None, pixel_format)?;
        for _ in 0..num_frames {
            ffmpeg_stdin.write_all(pixel_data)?;
        }
    }

    // Wait for the ffmpeg process to complete
    ffmpeg.wait()?;

    Ok(())
}
