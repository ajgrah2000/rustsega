use std::fs::{File};
use std::io::Read;

type BankSizeType = u16;
type NumBanksType = u8;

static NUM_RAMP_AGES:          u8 = 2;
const  BANK_SIZE:    BankSizeType = 0x4000;
const  MAX_BANKS:    NumBanksType = 64;
static LOWERMASK:             u16 = 0x03FFF;

#[derive(Copy, Clone)]
struct Bank
{
    data: [u8; BANK_SIZE as usize]
}

pub struct Cartridge
{
    filename: String,
    ram: Vec<Bank>,
    max_banks: u8,
    num_banks: NumBanksType,
    num_ram_pages: u8,
    rom: [Bank; MAX_BANKS as usize]
}

fn print(cartridge: &Cartridge) {
//    for i in 0..cartridge.num_banks as usize {
//        for b in cartridge.rom[i].data {
//            print!("{}", b);
//        }
//    }

    println!("");
    println!("read: {}", cartridge.filename);
    println!("Num banks: {}", cartridge.num_banks);
}

impl Cartridge {
    pub fn new (filename: &str) -> Self {
        Self {filename:filename.to_string(), ram:Vec::new(), max_banks:0, num_banks:0, num_ram_pages:0, rom:[Bank{data:[0; BANK_SIZE as usize]}; MAX_BANKS as usize]}
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        let mut file = File::open(&self.filename)?;
    
        (self.rom, self.num_banks) = read_banks(&mut file);
    
        self.ram = initialise_cartridge_ram(&MAX_BANKS);
    
        print(&self);
    
        Ok(())
    }
}

fn initialise_cartridge_ram(num_ram_banks: &NumBanksType) -> Vec<Bank> {
    let mut ram = Vec::new();
    for _i in 0..*num_ram_banks {
        ram.push(Bank{data:[0; BANK_SIZE as usize]});
    }
    ram 
}

fn read_banks(source: &mut dyn Read) -> ([Bank; MAX_BANKS as usize], NumBanksType) {
    let mut banks = [Bank{data:[0; BANK_SIZE as usize]}; MAX_BANKS as usize];
    let mut num_banks = 0;

    for i in 0..MAX_BANKS {
        match read_bank(source)
        {
            (Some(bank), _n) => {banks[i as usize] = bank;
                                num_banks = num_banks + 1},
            _ => {}
        }
    }

    (banks, num_banks)
}

fn read_bank(source: &mut dyn Read) -> (Option<Bank>, NumBanksType) {
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
