mod builder;
use builder::Builder;

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
        .get_matches();

    if matches.is_present("file") {
        match Builder::generate_single_file(matches.value_of("PATH").unwrap(), "out.html") {
            Ok(()) => println!("Wrote {} to out.html",
                           matches.value_of("PATH").unwrap()),
            Err(err) => panic!("{:?}", err)
        }
    } else {
        let mut builder = match Builder::new(matches.value_of("PATH").unwrap()) {
            Ok(builder) => builder,
            Err(error) => panic!("Couldn't load builder: {:?}", error)
        };

        if let Err(error) = builder.generate("generated", true) {
            panic!("Couldn't generate site: {:?}", error);
        } else {
            println!("Generation successful!");
        }
    }
}
