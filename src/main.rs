// While in progress, allow dead hode (at least until it's all hooked up)
#![allow(dead_code)]
#![allow(unused_variables)]
use std::env;

use argparse;

mod sega;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut debug = false;
    let mut cartridge_name = String::new();

    {
        // Handle command line arguments.
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Rusty Sega Emulator");
        ap.refer(&mut debug).add_option(&["-d","--debug"], argparse::StoreTrue, "Print PC State Debug Info");
        ap.refer(&mut cartridge_name).add_argument("cartridge", argparse::Store, "Name of cratridge to run").required();
        ap.parse_args_or_exit();
    }

    let mut cartridge = sega::memory::cartridge::Cartridge::new(&cartridge_name);
    match cartridge.load()
    {
        Ok(()) => {println!("Ok");}
        _ => {println!("Error loading cartridge.");}
    }

    let clock = sega::clocks::Clock::new();
    let mut memory = sega::memory::memory::MemoryAbsolute::new();
    let pc_state = sega::cpu::pc_state::PcState::new();
    let vdp = sega::graphics::vdp::VDP::new();
    let mut ports = sega::ports::Ports::new();
    let interruptor = sega::interruptor::Interruptor::new();

    // Add the graphics device to the list of ports.
    ports.add_device(Box::new(vdp));

    memory.set_cartridge(cartridge);
    let mut core = sega::cpu::core::Core::new(clock, memory, pc_state, ports, interruptor);

    for _i in 0..1500000 {
        core.step(debug);
    }

    println!("Finished.");
}
