use super::cartridge;

pub struct MemoryBase
{
}

pub struct MemoryShare
{
    ram: [u8;MemoryBase::RAM_SIZE as usize],

    // 'Page0' always contains bank 0 for the first 'PAGE0' bytes, then up
    // to the page boundary may contain a different bank. This array is used
    // to copy the 'mixed' banks to all possible 'page0' pages.

    page0_copies: [[u8;MemoryShare::BANK_SIZE as usize];MemoryShare::MAX_BANKS as usize],

    // Remember the last bank assignments per page, and only swap if they differ
    pages: [NumPagesType; MemoryShare::NUM_PAGES as usize],
    paging_register_page0: NumPagesType,
    paging_register_page1: NumPagesType,
    paging_register_page2: NumPagesType,
    paging_register_ram:   NumPagesType,

    // Location to store the page references.
    memory_shared_lookup: [[u8;MemoryShare::BANK_SIZE as usize]; MemoryShare::NUM_PAGES as usize],
    cartridge: cartridge::Cartridge,
}

impl MemoryBase {
    const MEMREGISTERS: AddressType = 0xFFFC;
    const ADDRESS_MASK: AddressType = 0xFFFF;
    const RAMMASK:      u16 = 0xDFFF;

    const RAM_SELECT_REGISTER:        AddressType = 0xFFFC;
    const PAGE0_BANK_SELECT_REGISTER: AddressType = 0xFFFD;
    const PAGE1_BANK_SELECT_REGISTER: AddressType = 0xFFFE;
    const PAGE2_BANK_SELECT_REGISTER: AddressType = 0xFFFF;

    const MAPCARTRAM:      u8  = 0x08;
    const PAGEOFRAM:       u8  = 0x04;
    const PAGEOFRAMBITPOS: u16 = 2;

    // Memory map offsets
    const PAGE0:      u16 = 0x400;  // 0 to Page0 offset always holds bank 0
    const PAGE1:      u16 = 0x4000;
    const PAGE2:      u16 = 0x8000;
    const RAM_OFFSET: u16 = 0xC000;
    const RAM_SIZE:   u16 = 0x2000; // Upper RAM is mirrored

    const MEMMAPSIZE: u32 = 0x10000;
    const UPPERSHIFT: u16 = 14;
    const LOWERMASK:  AddressType = 0x03FFF;

}

type BankSizeType = u16;
type NumBanksType = u8;
type NumPagesType = u8;
type AddressType  = u16;

impl MemoryShare {

    const BANK_SIZE:    BankSizeType = 0x4000;
    const MAX_BANKS:    NumBanksType = 64;
    const NUM_PAGES:    NumPagesType = 3;

    pub fn new (cartridge: cartridge::Cartridge) -> Self {
        Self {
            ram:[0;MemoryBase::RAM_SIZE as usize],
            page0_copies: [[0;MemoryShare::BANK_SIZE as usize];MemoryShare::MAX_BANKS as usize],
            pages: [0;MemoryShare::NUM_PAGES as usize],
            paging_register_page0: 0,
            paging_register_page1: 1,
            paging_register_page2: 2,
            paging_register_ram:   0,

            // Location to store the page references.
            memory_shared_lookup: [[0;MemoryShare::BANK_SIZE as usize]; MemoryShare::NUM_PAGES as usize],
            cartridge: cartridge,
            }
    }

    pub fn read(&self, address: AddressType) -> u8 {
        // """ Assumes 'address' is 'int' or accepts >> operator. """
        self.memory_shared_lookup[(address >> MemoryBase::UPPERSHIFT) as usize][(address & MemoryBase::LOWERMASK) as usize]
    }

    pub fn read_array(&self, address: AddressType, length: AddressType) -> Vec<u8> {
        let mut result = Vec::new();

        for i in 0..length {
            result.push(self.read(address + i));
        }

        result
    }
    
    pub fn read16(&self, address: AddressType) -> u16 {
        self.read(address) as u16 + (self.read(address + 1) << 8) as u16
    }

    pub fn write(&mut self, address: AddressType, data: u8) -> () {
        // Should check inputs, see which instructions/condition can overflow
        self.internal_write(address, data & 0xFF);
    }

    fn internal_write(&mut self, address: AddressType, data: u8) -> () {

        let address = address & MemoryBase::ADDRESS_MASK as u16; // ADDRESS_MASK
        let bank_address = address & MemoryBase::LOWERMASK as u16; // BANK_MASK

        if address >= MemoryBase::RAM_OFFSET as AddressType {
            // Memory control registers are written through to RAM
            self.ram[(address as usize) & (MemoryBase::RAM_SIZE - 1) as usize] = data;
            // Ram is 'half' size, so duplicate write to mirror section of array
            self.ram[((address as usize) & (MemoryBase::RAM_SIZE - 1) as usize) as usize + MemoryBase::RAM_SIZE as usize] = data;

            if address >= MemoryBase::MEMREGISTERS {
                // Should make these conditiona, 
                if address == MemoryBase::PAGE0_BANK_SELECT_REGISTER {
                    self.paging_register_page0 = data;
                    if self.pages[0] != data {
                        self.update_fixed_read_page0();
                        self.pages[0] = data;
                    }
                } else if address == MemoryBase::PAGE1_BANK_SELECT_REGISTER {
                    self.paging_register_page1 = data;
                    if self.pages[1] != data {
                        self.update_fixed_read_page1();
                        self.pages[1] = data;
                    }
                } else if (address == MemoryBase::RAM_SELECT_REGISTER) || 
                          (address == MemoryBase::PAGE2_BANK_SELECT_REGISTER) {
                    if address == MemoryBase::RAM_SELECT_REGISTER {
                        self.paging_register_ram = data;
                    } else if address == MemoryBase::PAGE2_BANK_SELECT_REGISTER {
                        self.paging_register_page2 = data;
                    }
    
                    if self.pages[2] != data {
                        self.update_fixed_read_page2_ram();
                        self.pages[2] = data;
                    }
                }
            }
        }
        else if (address < MemoryBase::RAM_OFFSET as AddressType) && (address >= MemoryBase::PAGE2 as AddressType) {
            let ram_select = self.paging_register_ram;
            if (ram_select & 0x8) != 0 { // page2_is_cartridge_ram
              let mut cartridge_ram_page: u8 = 0;
              if (ram_select & 0x4) != 0 {
                cartridge_ram_page = 1;
              }
      
              self.cartridge.write(cartridge_ram_page, bank_address, data);
            } else {
              println!("Warning writting to ROM address {}", address)
            }
        } else {
            println!("Warning to unexpected address {}", address)
        }
    }

    fn populate_shared_lookups(&mut self) -> () {
        for bank in 0..self.cartridge.num_banks {
            // Page '0'/'1' lookup
            for address in 0..MemoryBase::PAGE0 {
                bank_address = address & MemoryBase::LOWERMASK // BANK_MASK
                self.page0_copies[bank][bank_address] = self.cartridge.read(0, bank_address);
            }

            for address in MemoryBase::PAGE0 .. MemoryBase::PAGE1 {
                bank_address = address & MemoryBase::LOWERMASK // BANK_MASK
                self.page0_copies[bank][bank_address] = self.cartridge.read(bank, bank_address);
            }
        }

        // TODO: Check if copy makes sense for ram (rather than move/reference).
        self.memory_shared_lookup[3] = [self.ram.try_into().unwrap_or_else(|v: Vec<T>| panic(!"This didn't work"), self.ram.try_into().unwrap_or_else(|v: Vec<T>| panic(!"This didn't work")].concat();
    }

    fn update_fixed_read_page0(&mut self) -> () {
        let page0_bank = self.paging_register_page0;
        self.memory_shared_lookup[0] = self.page0_copies[page0_bank as usize].clone();
    }

    fn update_fixed_read_page1(&mut self) -> () {
        let page1_bank = self.paging_register_page1;
        self.memory_shared_lookup[1] = self.cartridge.read_rom_bank(page1_bank);
    }

    fn update_fixed_read_page2_ram(&mut self) -> () {
        let ram_select = self.paging_register_ram;

        if ram_select & MemoryBase::MAPCARTRAM != 0 { // page2_is_cartridge_ram

          let mut cartridge_ram_page = 0;
          if (ram_select & MemoryBase::PAGEOFRAM) != 0 {
            cartridge_ram_page = 1;
          } 
      
          self.memory_shared_lookup[2] = self.cartridge.read_ram_page(cartridge_ram_page);

        } else {
          let page2_bank = self.paging_register_page2;
          if page2_bank < self.cartridge.num_banks {
             self.memory_shared_lookup[2] = self.cartridge.read_rom_bank(page2_bank);
          } else {
             // Assume this is a 'transient' situation, force an error if
             // subsequently accessed.
             println!("Page 2: selection larger than cratridge. {}", page2_bank);
             self.memory_shared_lookup[2] = [0;MemoryShare::BANK_SIZE as usize];
          }
        }
    }
}

