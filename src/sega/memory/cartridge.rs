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
        let mut buffer;

        #[cfg(not(target_os = "emscripten"))]
        { 
            use std::fs::File;
            use std::io::Read;

            buffer = Vec::new();
            let mut file = File::open(&self.filename)?;
            file.read_to_end(&mut buffer)?;
        }

        #[cfg(target_os = "emscripten")]
        {
            buffer = include_bytes!("/tmp/test_file.rom").to_vec();
        }

        self.load_banks(&mut buffer);

        print(self);

        Ok(())
    }

    fn load_banks(&mut self, source: &mut Vec<u8>) {
        self.rom = Box::new(
            [Bank {
                data: [0; BANK_SIZE as usize],
            }; MAX_BANKS as usize],
        );

        for i in 0..MAX_BANKS {
            let (bank, n) = load_bank(source);

            self.rom[i as usize] = bank;
            source.drain(0..n as usize);
            self.num_banks += 1
        }
    }

    pub fn read(&mut self, bank: NumBanksType, bank_address: BankSizeType) -> u8 {
        self.rom[bank as usize].data[bank_address as usize]
    }
}

fn load_bank(source: &mut Vec::<u8>) -> (Bank, BankSizeType) {
    let mut bank = Bank {
        data: [0; BANK_SIZE as usize],
    };

    // Try to read an entire bank.
    if source.len() >= BANK_SIZE as usize {
        bank.data = source[0..BANK_SIZE as usize].try_into().unwrap();
        (bank, BANK_SIZE as BankSizeType)
    } else {
        let length = source.len();
        if length > 0 {
            bank.data = source[0..length].try_into().unwrap();
        }
        (bank, source.len() as BankSizeType)
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
