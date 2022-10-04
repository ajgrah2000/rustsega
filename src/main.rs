// While in progress, allow dead hode (at least until it's all hooked up)
#![allow(dead_code)]
#![allow(unused_variables)]
use std::env;

use argparse;

mod sega;

fn parse_args(debug: &mut bool, cartridge_name: &mut String) -> () {
    // Handle command line arguments.
    let mut ap = argparse::ArgumentParser::new();
    ap.set_description("Rusty Sega Emulator");
    ap.refer(debug).add_option(&["-d","--debug"], argparse::StoreTrue, "Print PC State Debug Info");
    ap.refer(cartridge_name).add_argument("cartridge", argparse::Store, "Name of cratridge to run").required();
    ap.parse_args_or_exit();
}

fn main() {
    let mut debug = false;
    let mut cartridge_name = String::new();

    parse_args(&mut debug, &mut cartridge_name);

    let mut sega_machine = sega::sega::Sega::new(debug, cartridge_name);
    
    sega_machine.power_sega();

    println!("Finished.");
}
