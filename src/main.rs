mod builder;
use builder::{Builder, Mode, PreviewMode, ReleaseMode};

fn execute<T: Mode>(matches: &clap::ArgMatches) -> Result<(), std::io::Error> {
    let builder = match Builder::new::<T>(
        matches.value_of("PATH").unwrap(),
        matches.value_of("theme").unwrap(),
        matches.value_of("output").unwrap(),
    ) {
        Err(err) => panic!("Unable to instantiate builder: {:?}", err),
        Ok(builder) => builder,
    };

    match if matches.is_present("file") {
        builder.generate_single_file::<T>(matches.value_of("file").unwrap())
    } else {
        builder.generate::<T>()
    } {
        Err(err) => {
            builder.clear_generated_files();
            panic!("Generation failed with `{:?}``", err);
        }
        Ok(()) => {
            println!("Generation successful!");
            builder.copy_generated_files()
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let matches = clap::App::new("Silas")
        .version("0.1")
        .author("Johannes Huwald <hey@jhuwald.com>")
        .about("The SSG for my blog at jhuwald.com")
        .arg(
            clap::Arg::with_name("PATH")
                .help("The path to the blog folder")
                .required(true),
        )
        .arg(
            clap::Arg::with_name("theme")
                .long("theme")
                .help("The path to the theme directory")
                .required(true)
                .default_value("theme")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("file")
                .long("file")
                .short("f")
                .help("Only generate a post for this file")
                .required(false)
                .takes_value(true),
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
        execute::<PreviewMode>(&matches)?
    } else {
        execute::<ReleaseMode>(&matches)?
    };
    Ok(())
}
