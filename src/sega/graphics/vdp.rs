use super::super::ports;

// Create a dummy VDP, to try out hooking into ports.
pub struct VDP {
    ram: Vec<u8>,
    current_address: u32,
}

impl VDP {
    pub fn new() -> Self {
        Self {
            ram: Vec::new(),
            current_address: 0,
        }
    }

    pub fn set_address(&mut self, value: u32) -> () {

    }
    pub fn get_address(&self) -> u32 {
        self.current_address
    }

    pub fn write_port_be(&mut self, data: u8) -> () {
        self.current_address = self.current_address + 1;
    }

    pub fn read_port_be(&mut self) -> u8 {
        self.current_address = self.current_address + 1;
        self.current_address as u8
    }

    pub fn write_port_bf(&mut self, data: u8) -> () {
        self.current_address = self.current_address + 1;
    }

    pub fn read_port_bf(&mut self) -> u8 {
        self.current_address = self.current_address + 1;
        self.current_address as u8
    }
}

impl ports::Device for VDP {
    fn port_read(&mut self, port_address: u8) -> u8 {
        match port_address & 0x1 {
            0x0 => {self.read_port_be()}
            0x1 => {self.read_port_bf()}
            _ => {0 /* Unhandled, just return 0 for now */}
        }
    }
    fn port_write(&mut self, port_address: u8, value:u8) -> () {
        match port_address & 0x1 {
            0x0 => {self.write_port_be(value);}
            0x1 => {self.write_port_bf(value);}
            _ => {}
        }
    }
}

