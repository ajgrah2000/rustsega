use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interuptor;
use super::super::ports;
use super::instructions;

pub struct Core {
    clock:      clocks::Clock,
    memory:     memory::MemoryAbsolute,
    pc_state:   pc_state::PcState,
    ports:      ports::Ports,
    interuptor: interuptor::Interuptor,
    
//    instruction_lookup = instruction_store.InstructionStore(self.clocks, self.pc_state, self.ports)
    
}

impl Core {
    const IRQIM1ADDR: u16 = 0x38;

    pub fn new(clock: clocks::Clock, 
           memory: memory::MemoryAbsolute, 
           pc_state: pc_state::PcState, 
           ports: ports::Ports,
           interuptor: interuptor::Interuptor) -> Self {
    
        Self {
            clock: clock,
            memory: memory,
            pc_state: pc_state,
            ports: ports,
            interuptor: interuptor,
        }
    }
    fn interupt(&mut self) -> () {
        if self.pc_state.get_iff1() == 1 {
            if self.pc_state.get_im() == 1 {
                self.pc_state.increment_sp(-1);
                self.memory.write(self.pc_state.get_sp(), self.pc_state.get_pc_high());
                self.pc_state.increment_sp(-1);
                self.memory.write(self.pc_state.get_sp(), self.pc_state.get_pc_low());
                self.pc_state.set_pc(&Core::IRQIM1ADDR);

                // Disable maskable interupts
                self.pc_state.set_iff1(0);
            } else {
                // TODO: Fix error messages/handling.
                println!("interupt mode not supported");
            }
        }
    }

    pub fn step(&mut self, debug: bool) -> (){
        // Start with 'expanded' version of step
        self.interuptor.set_cycle(self.clock.cycles);

        let op_code = self.memory.read(self.pc_state.get_pc());

        if debug {
//            println!("clock: {}, op_code: {:x}, pc: {}", self.clock.cycles, op_code, self.pc_state.get_pc());
            println!("{}", self.pc_state);
        }
        instructions::Instruction::execute(op_code, &mut self.clock, &mut self.memory, &mut self.pc_state, &mut self.ports, &mut self.interuptor);
    }
}

#[test]
fn test_core_creation() {
    let mut clock = clocks::Clock::new();
    let mut memory = memory::MemoryAbsolute::new();
    let mut pc_state = pc_state::PcState::new();
    let mut ports = ports::Ports::new();
    let mut interuptor = interuptor::Interuptor::new();
    let mut core = Core::new(clock, memory, pc_state, ports, interuptor);

    core.step(true);
    println!("{}", core.pc_state);
    core.step(true);
}

