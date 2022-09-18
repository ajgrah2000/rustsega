use super::cartridge;

///  Map the current 'pc' address to an 'absolute' address.  The
/// structure of the 'absolute' address is somewhat arbitrary, but  the
/// idea is to be fairly sensible with the layout to make the mapping
/// sensible and efficient.
/// 
/// segments:
/// 
/// cartridge - ROM - 0x4000 * 64 (0x3F)
/// cartridge - RAM - 0x4000 * 2
/// system    - RAM - 0x2000
/// 
/// 
/// ROM - 0x000000 - 0x0FFFFF
/// ROM - 0x100000 - 0x1FFFFF
/// RAM   0x200000 - 0x207FFF
/// RAM   0x208000 - 0x20A000
/// 
/// -> Total = 3F 0x42
/// 
/// mapped memory:
///     0x0000 - 0x03FF                     -> ROM (bank 0) (0x0000 - 0x03FF)
///     0x0400 - 0x3FFF + (0xFFFD(0-0x3F))          -> ROM (bank x) (0x0400 - 0x4000)
///     0x4000 - 0x7FFF + (0xFFFE(0-0x3F))          -> ROM (bank x) (0x0000 - 0x4000)
///     0x8000 - 0xBFFF + (0xFFFF(0-0x3F) + 0xFFFC(0x0C)) -> ROM (bank x) (0x0000 - 0x4000) or RAM (bank x)
/// - 11x0
///     0xC000 - 0xDFFF                     -> System RAM   (0x0000 - 0x2000)
///     0xE000 - 0xFFFF                     -> System RAM   (0x0000 - 0x2000) (mirror)
/// 
/// 
///     0x0000 - 0x03FF                     -> ROM (bank 0) (0x0000 - 0x03FF)
///     0x0400 - 0x3FFF + (0xFFFD)          -> ROM (bank x) (0x0400 - 0x4000)
/// 
///     0x4000 - 0x7FFF + (0xFFFE)          -> ROM (bank x) (0x0000 - 0x4000)
///     0x8000 - 0xBFFF + (0xFFFF + 0xFFFC) -> ROM (bank x) (0x0000 - 0x4000) or RAM (bank x)
/// 
///     0xC000 - 0xDFFF                     -> System RAM   (0x0000 - 0x2000)
///     0xE000 - 0xFFFF                     -> System RAM   (0x0000 - 0x2000) (mirror)
/// 
/// 
///     absolute = page[(address >> 13)] | address & 0x1FFF
/// 
/// 
///     | 6-bit - page 0| 6-bit - page 1| 6-bit - page 2 | 1-bit - ram/rom select | 1-bit - ram select | 3 - sub-page address | 13 - address
/// 
///     all bits -> 6 + 6 + 6 + 18
/// 

pub struct MemoryAbsoluteConstants {
}

pub struct MemoryBase {
}

pub struct MemoryAbsolute {
    // Ram is 'mirrored'
    ram: [u8;(MemoryBase::RAM_SIZE  * 2) as usize],

    // 'Page0' always contains bank 0 for the first 'PAGE0' bytes, then up
    // to the page boundary may contain a different bank. This array is used
    // to copy the 'mixed' banks to all possible 'page0' pages.

    page0_copies: [[u8;MemoryBase::BANK_SIZE as usize];MemoryBase::MAX_BANKS as usize],

    page_2: u8,
    ram_select: u8,

    upper_mappings: [AbsoluteAddressType; MemoryAbsoluteConstants::NUM_UPPER_MAPPINGS as usize],

    // Complete memory map
    memory_map:     [u8; max(MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE,
                             MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE) as usize]
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

    const BANK_SIZE:    BankSizeType = 0x4000;
    const MAX_BANKS:    NumBanksType = 64;
    const NUM_PAGES:    NumPagesType = 3;

}

type BankSizeType = u16;
type NumBanksType = u8;
type NumPagesType = u8;
type AddressType  = u16;
type AbsoluteAddressType = u32;

//const fn max<T: ~const PartialOrd + Copy>(a: T, b: T) -> T {
//    [a, b][(a < b) as usize]
//}
const fn max(a: AbsoluteAddressType, b: AbsoluteAddressType) -> AbsoluteAddressType {
    [a, b][(a < b) as usize]
}

impl MemoryAbsoluteConstants {
    const NUM_UPPER_MAPPINGS:         u8 =   8;

    const MAX_ROM_SIZE:               u32 = 0x100000;
    const ABSOLUTE_PAGE_0_ROM_OFFSET: AbsoluteAddressType = 0x000000;
    const ABSOLUTE_PAGE_X_ROM_OFFSET: AbsoluteAddressType = 0x100000;
    const ABSOLUTE_CART_RAM_OFFSET:   AbsoluteAddressType = 0x200000;
    const ABSOLUTE_SYS_RAM_OFFSET:    AbsoluteAddressType = 0x208000;
    const ABSOLUTE_SEGMENT_SIZE:      AbsoluteAddressType =   0x2000;
}

impl MemoryAbsolute {

    pub fn new () -> Self {

        // TODO: Improve auto increment the offsets
        let mut index = 0;
        fn new_segment(index: &mut u32) -> u32 {
            *index = *index + 1;
            MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + (MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE * *index)
        }

        Self {
            upper_mappings: [MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET,   
                             new_segment(&mut index),
                             new_segment(&mut index),
                             new_segment(&mut index),
                             new_segment(&mut index),
                             new_segment(&mut index),
                             MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET, 
                             MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET],

            // Complete memory map
            memory_map:      [0; max(MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE,
                                     MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE) as usize],
            page0_copies:  [[0;MemoryBase::BANK_SIZE as usize]; MemoryBase::MAX_BANKS as usize],
            page_2:  0,
            ram:   [0;(MemoryBase::RAM_SIZE * 2) as usize],
            ram_select:  0,
        }
    }

    pub fn get_absolute_address(&self, address: AddressType) -> AbsoluteAddressType {
        self.upper_mappings[(address >> 13) as usize] | (address & 0x1FFF) as AbsoluteAddressType
    }

    pub fn get_max_absolute_instruction_address(&self) -> AbsoluteAddressType {
        MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + MemoryAbsoluteConstants::MAX_ROM_SIZE
    }

     pub fn read(&self, address: AddressType) -> u8 {
        self.memory_map[(self.upper_mappings[(address >> 13) as usize] | (address & 0x1FFF) as AbsoluteAddressType) as usize]
     }

     pub fn read16(&self, address: AddressType) -> u16 {
         self.read(address) as u16 + ((self.read(address + 1)  as u16) << 8)
     }

     pub fn read_array(&self, address: AddressType, length: AddressType) -> Vec<u8> {
        let mut result = Vec::new();

        for i in 0..length {
            result.push(self.read(address + i));
        }

        result
     }

     pub fn write_multi(&mut self, dest: AddressType, src: AddressType, length: AddressType) -> () {
        // write multiple bytes to memory.

         match (src.checked_add(length), dest.checked_add(length)) {
             (Some(a), Some(b)) if (a as u32) < MemoryBase::MEMMAPSIZE && 
                                   (b as u32) < MemoryBase::MEMMAPSIZE => {
                        for i in 0..length {
                            self.write(dest+i, self.read(src+i));
                        }
                       },
             _ => {println!("WARNING: Write out of range {}",dest + length); }
         }
     }

     pub fn write(&mut self, address: AddressType, data: u8) -> () {
        // TODO: Should check inputs, see which instructions/condition can overflow
        self.private_write(address, data & 0xFF)
     }

     pub fn set_cartridge(&mut self, cartridge: cartridge::Cartridge) -> () {
        self.initialise_read(cartridge);
     }

     fn initialise_read(&mut self, cartridge: cartridge::Cartridge) -> () {
        // Un-optimised address translation, uses paging registers. 

        self.populate_absolute_memory_map(cartridge);
        self.write(0xFFFC, 0);
        self.write(0xFFFD, 0);
        self.write(0xFFFE, 1);
        self.write(0xFFFF, 2);
     }

     fn populate_absolute_memory_map(&mut self, mut cartridge: cartridge::Cartridge) -> () {
        for bank in 0..cartridge.num_banks as NumBanksType {
            // Page '0'/'1' lookup
            for address in 0..MemoryBase::PAGE0 as BankSizeType {
                let bank_address = address & MemoryBase::LOWERMASK; // BANK_MASK
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET  + AbsoluteAddressType::from(bank_address + BankSizeType::from(bank) * MemoryBase::BANK_SIZE)) as usize] =  cartridge.read(0, bank_address);
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET  + AbsoluteAddressType::from(bank_address + BankSizeType::from(bank) * MemoryBase::BANK_SIZE)) as usize] =  cartridge.read(bank, bank_address);
            }

            for address in MemoryBase::PAGE0..MemoryBase::PAGE1 as BankSizeType {
                let bank_address = address & MemoryBase::LOWERMASK; // BANK_MASK
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET  + AbsoluteAddressType::from(bank_address + BankSizeType::from(bank) * MemoryBase::BANK_SIZE)) as usize] =  cartridge.read(bank, bank_address);
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET  + AbsoluteAddressType::from(bank_address + BankSizeType::from(bank) * MemoryBase::BANK_SIZE)) as usize] =  cartridge.read(bank, bank_address);
            }
        }
     }

     fn private_write(&mut self, address: AddressType, data: u8) -> () {
        let address = address & MemoryBase::ADDRESS_MASK; // ADDRESS_MASK;

        if address >= MemoryBase::RAM_OFFSET {
            if address >= MemoryBase::MEMREGISTERS {
                // Should make these conditiona, 
                if address == MemoryBase::PAGE0_BANK_SELECT_REGISTER {
                    self.upper_mappings[0] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET + (MemoryBase::BANK_SIZE * data as BankSizeType) as AbsoluteAddressType;
                    self.upper_mappings[1] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + (MemoryBase::BANK_SIZE * data as BankSizeType) as AbsoluteAddressType + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
                }
                else if address == MemoryBase::PAGE1_BANK_SELECT_REGISTER {
                    self.upper_mappings[2] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + (MemoryBase::BANK_SIZE * data as BankSizeType) as AbsoluteAddressType;
                    self.upper_mappings[3] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + (MemoryBase::BANK_SIZE * data as BankSizeType) as AbsoluteAddressType + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
                }
                else if (address == MemoryBase::RAM_SELECT_REGISTER) || (address == MemoryBase::PAGE2_BANK_SELECT_REGISTER) {

                    if address == MemoryBase::RAM_SELECT_REGISTER {
                        self.ram_select = data;
                    }
                    else if address == MemoryBase::PAGE2_BANK_SELECT_REGISTER {
                        self.page_2     = data;
                    }

                    if 0 != self.ram_select & MemoryBase::MAPCARTRAM { // page2_is_cartridge_ram
                      // Cart RAM select.
                      if 0 != self.ram_select & MemoryBase::PAGEOFRAM {
                        self.upper_mappings[4] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET + (MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE * 2);
                        self.upper_mappings[5] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET + (MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE * 3);
                      }
                      else {
                        self.upper_mappings[4] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET;
                        self.upper_mappings[5] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
                      }
                    }
                    else {
                        self.upper_mappings[4] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + (MemoryBase::BANK_SIZE * self.page_2 as BankSizeType) as AbsoluteAddressType;
                        self.upper_mappings[5] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + (MemoryBase::BANK_SIZE * self.page_2 as BankSizeType) as AbsoluteAddressType + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
                    }
                }
            }
    
        }
        let absolute_address = self.get_absolute_address(address);
        if absolute_address >= std::cmp::min(MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET, MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET) {
            self.memory_map[absolute_address as usize] = data;
        }
     }

}
