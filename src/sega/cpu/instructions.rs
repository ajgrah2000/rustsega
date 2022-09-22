use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interuptor;
use super::super::ports;
use super::instruction_set;

pub struct Instruction {
}

impl Instruction {
    pub fn execute(op_code: u8, clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            0xcb => { Self::execute_cb(clock, memory, pc_state, ports, interuptor);}
            0xdb => { instruction_set::in_a_n(clock, memory, pc_state, ports);}
            0xdd => { Self::execute_dd(clock, memory, pc_state, ports, interuptor);}
            0xed => { Self::execute_ed(clock, memory, pc_state, ports, interuptor);}
            0xfd => { Self::execute_fd(clock, memory, pc_state, ports, interuptor);}
            0xf3 => { instruction_set::DI(clock, pc_state);}
        
            _ => {println!("Opcode not implemented: {:x}", op_code); }
        }
    } 

    // Extended instructions
    pub fn execute_cb(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            _ => {println!("Extended(0xCB) Opcode not implemented: {:x}", op_code); }
        }
    } 

    // Extended instructions
    pub fn execute_dd(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            _ => {println!("Extended(0xDD) Opcode not implemented: {:x}", op_code); }
        }
    } 
    // Extended instructions
    pub fn execute_fd(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            _ => {println!("Extended(0xFD) Opcode not implemented: {:x}", op_code); }
        }
    } 
    // Extended instructions
    pub fn execute_ed(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            0x56 => { instruction_set::im_1(clock, pc_state);} 
            _ => {println!("Extended(0xED) Opcode not implemented: {:x}", op_code); }
        }
    } 
}
