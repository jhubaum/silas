mod builder;
use builder::{Builder, Mode, PreviewMode, ReleaseMode};

fn execute<T: Mode>(matches: &clap::ArgMatches) {
    let builder = match Builder::new(matches.value_of("PATH").unwrap()) {
        Err(err) => panic!("Unable to instantiate builder: {:?}", err),
        Ok(builder) => builder,
    };

    if matches.is_present("file") {
        println!("Printing a single file is currently not supported");
    } else {
        match builder.generate::<T>(matches.value_of("output").unwrap(), true) {
            Err(err) => panic!("Unable to generate website: {:?}", err),
            Ok(()) => println!("Generation successful!"),
        }
    }
}

fn main() {
    let matches = clap::App::new("Silas")
        .version("0.1")
        .author("Johannes Huwald <hey@jhuwald.com>")
        .about("The SSG for my blog at jhuwald.com")
        .arg(
            clap::Arg::with_name("PATH")
                .help("The path to the blog folder, or, if --file is set, to the file")
                .required(true),
        )
        .arg(
            clap::Arg::with_name("file")
                .long("file")
                .short("f")
                .help("Generate the html output for a single org file")
                .required(false)
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("preview")
                .long("preview")
                .help("Render the blog in preview mode")
                .required(false)
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true)
                .default_value("generated")
                .help("The output directory for the SSG")
                .required(true),
        )
        .get_matches();

    if matches.is_present("preview") {
        execute::<PreviewMode>(&matches)
    } else {
        execute::<ReleaseMode>(&matches)
    };
}
