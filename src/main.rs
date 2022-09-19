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

        let mut clock = sega::clocks::Clock::new();
        let mut memory = sega::memory::memory::MemoryAbsolute::new();
        let mut pc_state = sega::cpu::pc_state::PcState::new();
        let mut ports = sega::ports::Ports::new();
        let mut interuptor = sega::interuptor::Interuptor::new();

        memory.set_cartridge(cartridge);
        let mut core = sega::cpu::core::Core::new(clock, memory, pc_state, ports, interuptor);
        let debug = true;

        core.step(debug);
        core.step(debug);
        core.step(debug);
        core.step(debug);
        core.step(debug);
        core.step(debug);

        println!("Finished.");
    }
}
