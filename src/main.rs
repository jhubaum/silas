mod builder;
use builder::{ReleaseBuilder, PreviewBuilder, Builder};
use builder::website::Website;
use std::path::Path;

fn execute<T: Builder>(matches: &clap::ArgMatches) -> Result<(), builder::GenerationError> {
    let output_folder_path = matches.value_of("output").unwrap();
    T::prepare_folder(output_folder_path, true)?;

    let builder = match T::new(output_folder_path) {
        Err(err) => panic!("Unable to load theme: {:?}", err),
        Ok(builder) => builder
    };

    let path = matches.value_of("PATH").unwrap();

    if matches.is_present("file") {
        match builder.generate_single_file(path, "out.html") {
            Ok(()) => println!("Wrote {} to out.html",
                           matches.value_of("PATH").unwrap()),
            Err(err) => panic!("{:?}", err)
        }
    } else {
        let website = Website::load(Path::new(path))?;

        builder.generate(&website)?;
    }
    Ok(())
}

fn main() {
    let matches = clap::App::new("Silas")
        .version("0.1")
        .author("Johannes Huwald <hey@jhuwald.com>")
        .about("The SSG for my blog at jhuwald.com")
        .arg(clap::Arg::with_name("PATH")
             .help("The path to the blog folder, or, if --file is set, to the file")
             .required(true))
        .arg(clap::Arg::with_name("file")
             .long("file")
             .short("f")
             .help("Generate the html output for a single org file")
             .required(false)
             .takes_value(false))
        .arg(clap::Arg::with_name("preview")
             .long("preview")
             .help("Render the blog in preview mode")
             .required(false)
             .takes_value(false))
        .arg(clap::Arg::with_name("output")
             .long("output")
             .short("o")
             .takes_value(true)
             .default_value("generated")
             .help("The output directory for the SSG")
             .required(true))
        .get_matches();

    builder::debug_new_website(matches.value_of("PATH").unwrap());
    return;

    let res = if matches.is_present("preview") {
        execute::<PreviewBuilder>(&matches)
    } else {
        execute::<ReleaseBuilder>(&matches)
    };

    match res {
        Ok(()) => println!("Generation successful!"),
        Err(err) => panic!("Couldn't generate site: {:?}", err)
    };
}
