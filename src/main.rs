// While in progress, allow dead hode (at least until it's all hooked up)
#![allow(dead_code)]
#![allow(unused_variables)]

mod sega;

use argh::FromArgs;

#[derive(FromArgs)]
/// Rusty Sega Emulator.
struct RustSegaArgs {
    /// print PC State Debug Info
    #[argh(switch, short='d')]
    debug: bool,

    /// run the emulator with no delay (rather than real-time)
    #[argh(switch, short='n')]
    no_delay: bool,

    /// number of clock cycles to stop the emulator (for benchmarking)
    #[argh(option, short='s')]
    stop_clock: Option<u64>,

    /// run the emulator in full screen mode.
    #[argh(switch, short='f')]
    fullscreen: bool,

    /// list SDL drivers
    #[argh(switch, short='l')]
    list_drivers: bool,

    /// name of cartridge to run
    #[argh(positional)]
    cartridge_name: String,
}

fn full_description_string() -> String {
    let mut description = "Possible audio drivers, to use prefix command with: SDL_AUDIODRIVER=<driver>\n".to_owned();
    description += &sdl2::audio::drivers().map(|s| s.to_string()).reduce(|cur: String, nxt:String| cur + ", " + &nxt).unwrap();
    description += "\n";
    description += "Possible video drivers, to use prefix command with: SDL_VIDEODRIVER=<driver>\n";
    description += &sdl2::video::drivers().map(|s| s.to_string()).reduce(|cur: String, nxt:String| cur + ", " + &nxt).unwrap();
    description += "\n";

    description.to_string()
}

fn main() {

    let args:RustSegaArgs = argh::from_env();

    if args.list_drivers {
        println!("{}", full_description_string());
    }
    let mut sega_machine = sega::sega::Sega::new(args.debug, !args.no_delay, args.stop_clock.unwrap_or(0), args.cartridge_name, args.fullscreen);

    sega_machine.power_sega();

    println!("Finished.");
}
