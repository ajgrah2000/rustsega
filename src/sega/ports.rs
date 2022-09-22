struct NullPort {
}

impl NullPort {
    fn new() -> Self {
        Self {}
    }
}

enum PortEnum {
    NullPort, 
}

pub trait Port {
    fn write(&mut self, value: u8) -> (); 
    fn read(&mut self) -> u8; 
}

impl Port for NullPort {
    fn write(&mut self, value: u8) -> () {
        println!("null write value = {}", value);
    }

    fn read(&mut self) -> u8 {
        0
    }
}

pub struct Ports {
    devices: Vec<Box<dyn Port>>,
}

impl Ports {
    const MAXPORTS: u16 = 256;
    pub fn new() -> Self {
        let mut new_devices: Vec<Box<dyn Port>> = Vec::new();
        for _i in 0..Ports::MAXPORTS {
            let new_port = NullPort::new();
            new_devices.push(Box::new(new_port));
        }
        Self {
            devices: new_devices,
        }
    }

    pub fn add_port(&mut self, port_address: u8, port: Box<dyn Port>) -> () {
        self.devices[port_address as usize] = port;
    }

    pub fn port_read(&mut self, port_address: u8) -> u8 {
        self.devices[port_address as usize].read()
    }

    pub fn port_write(&mut self, port_address: u8, value:u8) -> () {
        self.devices[port_address as usize].write(value)
    }

    fn port_multi_write(&mut self, port_address: u8, data: Vec<u8>) -> () {
        for value in data {
            self.port_write(port_address, value)
        }
    }
}

