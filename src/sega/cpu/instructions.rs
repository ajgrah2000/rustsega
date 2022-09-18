use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interuptor;

struct Instruction {
}

impl Instruction {
    fn execute(op_code: u8, clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           interuptor: &mut interuptor::Interuptor) {
        match op_code {
            0 => { pc_state.increment_pc(1); } // Noop
            _ => {}
        }
    } 
}
