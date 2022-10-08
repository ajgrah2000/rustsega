use std::fs::File;
use std::io::Read;

type BankSizeType = u16;
type NumBanksType = u8;

const BANK_SIZE: BankSizeType = 0x4000;
const MAX_BANKS: NumBanksType = 64;

#[derive(Copy, Clone)]
struct Bank {
    data: [u8; BANK_SIZE as usize],
}

pub struct Cartridge {
    filename: String,
    pub num_banks: NumBanksType,
    rom: Box<[Bank; MAX_BANKS as usize]>,
}

fn print(cartridge: &Cartridge) {
    println!("read: {}", cartridge.filename);
    println!("Num banks: {}", cartridge.num_banks);
}

impl Cartridge {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            num_banks: 0,
            rom: Box::new(
                [Bank {
                    data: [0; BANK_SIZE as usize],
                }; MAX_BANKS as usize],
            ),
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        let mut file = File::open(&self.filename)?;

        self.load_banks(&mut file);

        print(self);

        Ok(())
    }

    fn load_banks(&mut self, source: &mut dyn Read) {
        self.rom = Box::new(
            [Bank {
                data: [0; BANK_SIZE as usize],
            }; MAX_BANKS as usize],
        );

        for i in 0..MAX_BANKS {
            if let (Some(bank), _n) = load_bank(source) {
                self.rom[i as usize] = bank;
                self.num_banks += 1
            }
        }
    }

    pub fn read(&mut self, bank: NumBanksType, bank_address: BankSizeType) -> u8 {
        self.rom[bank as usize].data[bank_address as usize]
    }
}

fn load_bank(source: &mut dyn Read) -> (Option<Bank>, NumBanksType) {
    let mut bank = Bank {
        data: [0; BANK_SIZE as usize],
    };

    // Try to read an entire bank.
    match source.read(&mut bank.data) {
        Ok(0) => (None, 0),
        Ok(n) if n < BANK_SIZE as usize => {
            println!(
                "Bank incomplete ({} bytes found in last bank), will be padded with zeros",
                n
            );
            (Some(bank), n as NumBanksType)
        }
        Ok(n) => (Some(bank), n as NumBanksType),
        _ => (None, 0),
    }
}

#[cfg(test)]
mod tests {
    use crate::sega::memory::cartridge::Cartridge;
    use std::mem;

    #[test]
    fn test_load_rom() {
        // Do a test load of a 'fake rom' (just randomly generated data).
        let test_rom = "fake.rom";
        let mut cartridge = Cartridge::new(test_rom);
        match cartridge.load() {
            Ok(()) => {
                println!("Ok");
            }
            _ => {
                println!("Error loading cartridge.");
            }
        }
        assert_eq!(cartridge.read(0, 0), 139);
        println!("{}", mem::size_of_val(&cartridge));
    }
}
