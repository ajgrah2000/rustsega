// While in progress, allow dead hode (at least until it's all hooked up)
#![allow(dead_code)]
#![allow(unused_variables)]
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

        let clock = sega::clocks::Clock::new();
        let mut memory = sega::memory::memory::MemoryAbsolute::new();
        let pc_state = sega::cpu::pc_state::PcState::new();
        let ports = sega::ports::Ports::new();
        let interuptor = sega::interuptor::Interuptor::new();

        memory.set_cartridge(cartridge);
        let mut core = sega::cpu::core::Core::new(clock, memory, pc_state, ports, interuptor);

        let debug = true;

        for _i in 0..10 {
            core.step(debug);
        }

        println!("Finished.");
    }
}
