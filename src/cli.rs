use std::{env, process};

use lexopt::Parser;

use crate::bail;
use crate::err::Result;

fn print_help() {
    println!(
        "{}",
        format!(
            r#"
{crate_name} {crate_version}
{crate_authors}

Project home page: {crate_homepage}

USAGE:
    {bin} [OPTIONS] <INPUT_FILE>

INPUT_FILE:
    This should be one of:
        - Float Control CSV or ZIP
        - Floaty JSON

REQUIRED FLAGS:
    -c, --cell-count <COUNT>         Number of cells in the battery pack
    -f, --font <FONT>                Path to the font file (TTF) to use for rendering text

OPTIONAL FLAGS:
    -g, --max-gap-seconds <SECONDS>  Maximum gap between data points (in seconds) [default: 2.0]
    -o, --output <OUTPUT>            Output file name [default: $input_file_name.mov]
    -r, --rate <FRAME_RATE>          Frame rate of the output video [default: 30]
    -s, --scale <SCALE>              Scale factor for the output video [default: 1.0]
    -t, --title-font <TITLE_FONT>    Path to the font file (TTF) to use for rendering titles [default: FONT]
    -T, --transparent                Encode with a transparent background - note that due to encoding
                                     formats, enabling this significantly increases file size [default: false]


    -h, --help                       Print help information
    -V, --version                    Print version information

EXAMPLES:
    {bin}                  path/to/float-control.csv
    {bin} --scale 1.2      path/to/float-control.csv
    {bin} --rate 60        path/to/floaty.json
    {bin} --output vid.mov path/to/floaty.json

    "#,
            bin = env!("CARGO_BIN_NAME"),
            crate_name = env!("CARGO_PKG_NAME"),
            crate_version = env!("CARGO_PKG_VERSION"),
            crate_homepage = env!("CARGO_PKG_HOMEPAGE"),
            crate_authors = env!("CARGO_PKG_AUTHORS")
                .split(':')
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .trim(),
    );
}

#[derive(Debug)]
pub struct Args {
    pub input: String,
    pub output: String,
    pub max_gap_seconds: f32,
    pub cell_count: u8,
    pub rate: f32,
    pub scale: f32,
    pub font: String,
    pub title_font: String,
    pub transparent_bg: bool,
}

impl Args {
    pub fn parse() -> Result<Args> {
        use lexopt::prelude::*;

        let mut input = None;

        let mut max_gap_seconds = None;
        let mut cell_count = None;
        let mut rate = None;
        let mut output = None;
        let mut font = None;
        let mut title_font = None;
        let mut scale = None;
        let mut transparent_bg = false;

        let mut parser = Parser::from_env();
        while let Some(arg) = parser.next()? {
            match arg {
                Short('s') | Long("scale") => scale = Some(parser.value()?.string()?.parse()?),
                Short('f') | Long("font") => font = Some(parser.value()?.string()?.into()),
                Short('T') | Long("transparent") => transparent_bg = true,
                Short('t') | Long("title-font") => {
                    title_font = Some(parser.value()?.string()?.into())
                }
                Short('r') | Long("rate") => rate = Some(parser.value()?.string()?.parse()?),
                Short('o') | Long("output") => output = Some(parser.value()?.string()?.into()),
                Short('c') | Long("cell-count") => {
                    cell_count = Some(parser.value()?.string()?.parse()?)
                }
                Short('g') | Long("max-gap-seconds") => {
                    max_gap_seconds = Some(parser.value()?.string()?.parse()?)
                }
                Short('h') | Long("help") => {
                    print_help();
                    process::exit(0);
                }
                Short('v') | Long("version") => {
                    println!(
                        "{crate_name} {crate_version}",
                        crate_name = env!("CARGO_PKG_NAME"),
                        crate_version = env!("CARGO_PKG_VERSION")
                    );
                    process::exit(0);
                }
                Value(val) if input.is_none() => {
                    input = Some(val.string()?.into());
                }
                Short(_) | Long(_) | Value(_) => {
                    print_help();
                    process::exit(1);
                }
            }
        }

        if input.is_none() {
            bail!("no input file specified");
        }

        if cell_count.is_none() {
            print_help();
            bail!("cell count is required");
        }

        if font.is_none() {
            print_help();
            bail!("font is required");
        }

        Ok(Args {
            input: input.unwrap(),
            output: output.unwrap_or(String::from("output.mov")),
            max_gap_seconds: max_gap_seconds.unwrap_or(2.0),
            cell_count: cell_count.unwrap(),
            title_font: title_font.unwrap_or_else(|| font.clone().unwrap()),
            font: font.unwrap(),
            rate: rate.unwrap_or(30.0),
            scale: scale.unwrap_or(1.0),
            transparent_bg,
        })
    }
}
