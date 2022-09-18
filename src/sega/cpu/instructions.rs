use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interuptor;

pub struct Instruction {
}

impl Instruction {
    pub fn execute(op_code: u8, clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           interuptor: &mut interuptor::Interuptor) -> () {
        match op_code {
            0x00 => { pc_state.increment_pc(1); println!("{}", pc_state);} // Noop
            0xf3 => { pc_state.increment_pc(1); println!("{}", pc_state);} // TODO: Implement actual instruction
        
            _ => {}
        }
    } 
}
