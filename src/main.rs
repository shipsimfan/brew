use std::{env::args, process::exit};

mod arguments;
mod brewfile;
mod config;
mod parser;

fn print_error(error: Box<dyn std::error::Error>) -> ! {
    println!("\x1B[31;1mError:\x1B[0m {}", error);
    exit(1);
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => print_error(error),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let options = arguments::parse_arguments(args().collect())?;
    if options.verbose() {
        println!("Options");
        println!("========================================");
        print!("{}", options);
    }

    // Parse the brewfile
    let brewfile = parser::parse_brewfile()?;
    if options.verbose() {
        println!();
        println!("Brewfile");
        println!("========================================");
        print!("{}", brewfile);
    }

    // Execute the brewfile
    brewfile.execute(options)
}
