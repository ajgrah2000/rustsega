use std::env;

mod sega;

fn usage() {
    println!("Usage: <prog> <rom>");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        usage();
    } else{
        let filename = &args[1];

        match sega::memory::cartridge::load(filename)
        {
            Ok(()) => {println!("Ok");}
            _ => {println!("Error loading cartridge.");}
        }
        println!("Finished.");
    }
}
