// While in progress, allow dead hode (at least until it's all hooked up)
#![allow(dead_code)]
#![allow(unused_variables)]

mod sega;

use clap::Parser;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct RustSegaArgs {
    #[arg(short, long, help="Print PC State Debug Info")]
    debug: bool,
    #[arg(short, long, action=clap::ArgAction::SetTrue, help="Run the emulator with no delay (rather than real-time)")]
    no_delay: bool,
    #[arg(short, long, default_value_t = 0, help="Number of clock cycles to stop the emulator (for benchmarking)")]
    stop_clock: u64,
    #[arg(short, long, help="Run the emulator in full screen mode.")]
    fullscreen: bool,
    #[arg(short, long, help="List SDL drivers")]
    list_drivers: bool,
    #[arg(help="Name of cartridge to run")]
    cartridge_name: String,
}

fn full_description_string() -> String {
    let mut description = format!("Rusty Sega Emulator. ");
    description += &format!("Possible audio drivers, to use prefix command with: SDL_AUDIODRIVER=<driver> ");
    for i in sdl2::audio::drivers() {
        description += &(format!("{}", i) + "\n");
    }

    description += "\n";
    description += &format!("Possible video drivers, to use prefix command with: SDL_VIDEODRIVER=<driver> ");
    for i in sdl2::video::drivers() {
        description += &(format!("{}", i) + "\n");
    }

    description
}

fn main() {

    let args = RustSegaArgs::parse();

    if args.list_drivers {
        println!("{}", full_description_string());
    }
    let mut sega_machine = sega::sega::Sega::new(args.debug, !args.no_delay, args.stop_clock, args.cartridge_name, args.fullscreen);

    sega_machine.power_sega();

    println!("Finished.");
}
