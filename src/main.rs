use std::env::args;

mod builder;
use builder::Builder;

fn main() {
    let args: Vec<String> = args().collect();
    let mut builder = match Builder::new(&args[1]) {
        Ok(builder) => builder,
        Err(error) => panic!("Couldn't load builder: {:?}", error)
    };

    if let Err(error) = builder.generate("generated", true) {
        panic!("Couldn't generate site: {:?}", error);
    } else {
        println!("Generation successful!");
    }
}
