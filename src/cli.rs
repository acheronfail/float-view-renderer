use std::{env, process};

use anyhow::{bail, Result};
use lexopt::Parser;

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

OPTIONS:
    -s, --scale <SCALE>              Scale factor (controls size of the output) [default: 8.0]
    -r, --rate <FRAME_RATE>          Frame rate of the output video (when unset, a variable frame rate is used)
    -o, --output <OUTPUT>            Output file name [default: $input_file_name.mov]
    -c, --cell-count <COUNT>         Number of cells in the battery pack
    -g, --max-gap-seconds <SECONDS>  Maximum gap between data points (in seconds) [default: 2.0]
    -h, --help                       Print help information
    -V, --version                    Print version information

EXAMPLES:
    {bin}                  path/to/float-control.csv
    {bin} --scale 10.0     path/to/float-control.csv
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
    pub scale: f32,
    pub max_gap_seconds: f32,
    pub cell_count: Option<u8>,
    pub rate: Option<f32>,
}

impl Args {
    pub fn parse() -> Result<Args> {
        use lexopt::prelude::*;

        let mut input = None;

        let mut max_gap_seconds = None;
        let mut cell_count = None;
        let mut scale = None;
        let mut rate = None;
        let mut output = None;

        let mut parser = Parser::from_env();
        while let Some(arg) = parser.next()? {
            match arg {
                Short('s') | Long("scale") => scale = Some(parser.value()?.string()?.parse()?),
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

        Ok(Args {
            input: input.unwrap(),
            scale: scale.unwrap_or(8.0),
            output: output.unwrap_or(String::from("output.mov")),
            max_gap_seconds: max_gap_seconds.unwrap_or(2.0),
            cell_count,
            rate,
        })
    }
}
