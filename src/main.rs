// While in progress, allow dead hode (at least until it's all hooked up)
#![allow(dead_code)]
#![allow(unused_variables)]

mod sega;

fn parse_args(debug: &mut bool, realtime: &mut bool, stop_clock: &mut u64, cartridge_name: &mut String) {
    // Handle command line arguments.
    let mut ap = argparse::ArgumentParser::new();
    ap.set_description("Rusty Sega Emulator");
    ap.refer(debug).add_option(
        &["-d", "--debug"],
        argparse::StoreTrue,
        "Print PC State Debug Info",
    );
    ap.refer(stop_clock).add_option(
        &["-s", "--stop_clock"],
        argparse::Store,
        "Number of clock cycles to stop the emulator (for benchmarking)",
    );
    ap.refer(realtime).add_option(
        &["-n", "--no_delay"],
        argparse::StoreFalse,
        "Run the emulator with no delay (rather than real-time)",
    );
    ap.refer(cartridge_name)
        .add_argument("cartridge", argparse::Store, "Name of cartridge to run")
        .required();
    ap.parse_args_or_exit();
}

fn main() {
    let mut debug = false;
    let mut realtime = true;
    let mut stop_clock = 0;
    let mut cartridge_name = String::new();

    parse_args(&mut debug, &mut realtime, &mut stop_clock, &mut cartridge_name);

    let mut sega_machine = sega::sega::Sega::new(debug, realtime, stop_clock, cartridge_name);

    sega_machine.power_sega();

    println!("Finished.");
}
