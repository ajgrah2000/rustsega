use super::memory::memory;
use super::cpu::pc_state;
use super::cpu::core;
use super::clocks;

pub struct Interruptor {
    pub next_interrupt: u32
}

pub trait Interrupt {
    fn interrupt<M>(pc_state: &mut pc_state::PcState, memory: &mut M) -> () where M: memory::MemoryRW;
}

pub trait PollForInterrupt {
    fn poll_interrupts(&mut self, clock: &clocks::Clock) -> bool;
}

impl Interruptor {
    pub fn interrupt<M>(pc_state: &mut pc_state::PcState, memory: &mut M) -> () where M: memory::MemoryRW
    {
        if pc_state.get_iff1() {
            if pc_state.get_im() == 1 {
                pc_state.increment_sp(-1);
                memory.write(pc_state.sp_reg.get(), pc_state.get_pc_high());
                pc_state.increment_sp(-1);
                memory.write(pc_state.sp_reg.get(), pc_state.get_pc_low());
                pc_state.set_pc(core::Core::<M>::IRQIM1ADDR as u16);

                // Disable mask-able interrupts
                pc_state.set_iff1(false);
            }
            else
            {
                panic!("Unsupported interrupt mode: {}",  pc_state.get_im());
            }
        }
    }
}

impl Interruptor {
    pub fn new () -> Self {
        Self {
            next_interrupt: 0,
        }
    }
    // TODO: Add the actual interruptor trait/implementation (previously VDU).
    pub fn set_cycle(&mut self, cycles: u32) -> () {
        // TODO: Do something

    }
}
