use std::fs::{File};
use std::io::Read;
use std::mem;

type BankSizeType = u16;
type NumBanksType = u8;

const NUM_RAM_PAGES:          u8 = 2;
const BANK_SIZE:    BankSizeType = 0x4000;
const MAX_BANKS:    NumBanksType = 64;
const LOWERMASK:             u16 = 0x03FFF;

#[derive(Copy, Clone)]
struct Bank
{
    data: [u8; BANK_SIZE as usize]
}

pub struct Cartridge
{
    filename: String,
    ram: [[u8; BANK_SIZE as usize]; NUM_RAM_PAGES as usize],
    pub num_banks: NumBanksType,
    num_ram_pages: u8,
    rom: Box<[Bank; MAX_BANKS as usize]>,
}

fn print(cartridge: &Cartridge) {
    println!("read: {}", cartridge.filename);
    println!("Num banks: {}", cartridge.num_banks);
}

impl Cartridge {
    pub fn new (filename: &str) -> Self {
        Self {filename:filename.to_string(), 
              ram: [[0; BANK_SIZE as usize]; NUM_RAM_PAGES as usize],
              num_banks:0, 
              num_ram_pages:0, 
              rom:Box::new([Bank{data:[0; BANK_SIZE as usize]}; MAX_BANKS as usize]),
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        let mut file = File::open(&self.filename)?;
    
         self.load_banks(&mut file);
    
        print(&self);
    
        Ok(())
    }

//    pub fn read(&self, address: &BankSizeType) -> u8 {
//        self.rom[self.current_bank as usize].data[(*address & LOWERMASK) as usize]
//    }

    fn load_banks(&mut self, source: &mut dyn Read) {
        self.rom = Box::new([Bank{data:[0; BANK_SIZE as usize]}; MAX_BANKS as usize]);
    
        for i in 0..MAX_BANKS {
            match load_bank(source)
            {
                (Some(bank), _n) => {self.rom[i as usize] = bank; self.num_banks = self.num_banks + 1},
                _ => {}
            }
        }
    }

    pub fn write(&mut self, page: u8, address: BankSizeType, data: u8) -> () {
        self.ram[page as usize][address as usize] = data;
    }

    pub fn read(&mut self, bank: NumBanksType, bank_address: BankSizeType ) -> u8 {
        self.rom[bank as usize].data[bank_address as usize]
    }

    pub fn read_ram_page(&mut self, ram_page: u8) -> [u8; BANK_SIZE as usize] {
        self.ram[ram_page as usize].clone()
    }

    pub fn read_rom_bank(&mut self, rom_bank: u8) -> [u8; BANK_SIZE as usize] {
        self.rom[rom_bank as usize].data.clone()
    }

}


fn load_bank(source: &mut dyn Read) -> (Option<Bank>, NumBanksType) {
    let mut bank = Bank {data: [0; BANK_SIZE as usize]};

    // Try to read an entire bank.
    match source.read(&mut bank.data)
    {
        Ok(0) => {(None, 0)},
        Ok(n) if n < BANK_SIZE as usize => {
            println!("Bank incomplete ({} bytes found in last bank), will be padded with zeros", n);
           (Some(bank), n as NumBanksType)},
        Ok(n) => {(Some(bank), n as NumBanksType)},
        _ => {(None, 0)}
    }
}

#[test]
fn test_load_rom() {
    // Do a test load of a 'fake rom' (just randomly generated data).
    let test_rom = "fake.rom";
    let mut cartridge = Cartridge::new(test_rom);
    cartridge.load(); 
    assert_eq!(cartridge.read(0, 0), 139);
    println!("{}", mem::size_of_val(&cartridge));
}
