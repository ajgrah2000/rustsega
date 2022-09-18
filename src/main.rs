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

        let mut cartridge = sega::memory::cartridge::Cartridge::new(filename);
        match cartridge.load()
        {
            Ok(()) => {println!("Ok");}
            _ => {println!("Error loading cartridge.");}
        }

        let mut memory = sega::memory::memory::MemoryAbsolute::new();
        memory.set_cartridge(cartridge);

        println!("Finished.");
    }
}
