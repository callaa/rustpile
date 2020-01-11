use clap::{App, Arg};
use tracing::Level;
use tracing_subscriber;

use drawpile_cli::converter::*;
use drawpile_cli::renderer::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    let matches = App::new("Drawpile-cli")
        .version("0.1.0")
        .about("Convert Drawpile recordings")
        .subcommand(
            App::new("convert")
                .arg(Arg::with_name("INPUT").help("Input file").required(true))
                .arg(Arg::with_name("OUTPUT").help("Output file"))
                .arg(
                    Arg::with_name("format")
                        .short("f")
                        .long("format")
                        .help("Output format")
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("render").arg(Arg::with_name("INPUT").help("Input file").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        ("convert", Some(m)) => {
            let opts = ConvertRecOpts {
                input_file: m.value_of("INPUT").unwrap(),
                output_file: m.value_of("OUTPUT").unwrap_or("-"),
                output_format: Format::Guess,
            };

            convert_recording(&opts)
        }
        ("render", Some(m)) => {
            let opts = RenderOpts {
                input_file: m.value_of("INPUT").unwrap(),
            };

            render_recording(&opts)
        }
        ("", None) => {
            println!("Use: drawpile-cli convert or drawpile-cli render");
            Ok(())
        }
        _ => unreachable!(),
    }
}
