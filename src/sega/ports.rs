use super::clocks;

struct NullPort {}

impl NullPort {
    fn new() -> Self {
        Self {}
    }
}

pub trait Port {
    fn write(&mut self, clock: &clocks::Clock, value: u8);
    fn read(&mut self, clock: &clocks::Clock) -> u8;
}

pub trait Device {
    fn poll_interrupts(&mut self, raw_display: &mut Vec<u8>, clock: &clocks::Clock) -> bool;
    fn port_write(&mut self, clock: &clocks::Clock, port_address: u8, value: u8);
    fn port_read(&mut self, clock: &clocks::Clock, port_address: u8) -> u8;
    fn export(&mut self, raw_display: &mut Vec<u8>);
}

impl Port for NullPort {
    fn write(&mut self, _clock: &clocks::Clock, value: u8) {
        println!("null write value = {}", value);
    }

    fn read(&mut self, _clock: &clocks::Clock) -> u8 {
        0
    }
}

pub struct Ports {
    ports: Vec<Box<dyn Port>>,
    devices: Vec<Box<dyn Device>>,
}

impl Ports {
    const MAXPORTS: u16 = 256;
    pub fn new() -> Self {
        let mut new_ports: Vec<Box<dyn Port>> = Vec::new();
        for _i in 0..Ports::MAXPORTS {
            let new_port = NullPort::new();
            new_ports.push(Box::new(new_port));
        }
        Self {
            ports: new_ports,
            devices: Vec::new(),
        }
    }

    pub fn add_device(&mut self, device: Box<dyn Device>) {
        self.devices.push(device);
    }

    pub fn add_port(&mut self, port_address: u8, port: Box<dyn Port>) {
        self.ports[port_address as usize] = port;
    }

    pub fn port_read(&mut self, clock: &clocks::Clock, port_address: u8) -> u8 {
        // TODO: Fix dummy joystick return values.
        if port_address == 0xdc {
            return 0xff;
        }
        if port_address == 0xdd {
            return 0xff;
        }

        let mut last_value = 0;
        // TODO: Replace with something useful, use a map or lookup, or hook up ports directly.
        for i in 0..self.devices.len() {
            last_value = self.devices[i].port_read(clock, port_address)
        }
        last_value
    }

    pub fn port_write(&mut self, clock: &clocks::Clock, port_address: u8, value: u8) {
        for i in 0..self.devices.len() {
            // TODO: Replace with something useful.
            self.devices[i].port_write(clock, port_address, value);
        }
    }

    pub fn export(&mut self, raw_display: &mut Vec<u8>) {
        for i in 0..self.devices.len() {
            self.devices[i].export(raw_display);
        }
    }

    pub fn poll_interrupts(&mut self, raw_display: &mut Vec<u8>, clock: &clocks::Clock) -> bool {
        let mut interrupt = false;
        for i in 0..self.devices.len() {
            interrupt |= self.devices[i].poll_interrupts(raw_display, clock);
        }

        interrupt
    }
}
