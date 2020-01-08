use clap::{App, Arg};

use drawpile_cli::converter::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Drawpile-cli")
        .version("0.1.0")
        .about("Convert Drawpile recordings")
        .arg(Arg::with_name("INPUT").help("Input file").required(true))
        .arg(Arg::with_name("OUTPUT").help("Output file"))
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .help("Output format")
                .takes_value(true),
        )
        .get_matches();

    let convert = ConvertRecOpts {
        input_file: matches.value_of("INPUT").unwrap(),
        output_file: matches.value_of("OUTPUT").unwrap_or("-"),
        output_format: Format::Guess,
    };

    convert_recording(&convert)
}
