use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interruptor;
use super::super::ports;
use super::super::graphics;
use super::instructions;

pub struct Core<M> {
    clock:      clocks::Clock,
    memory:     M,
    pc_state:   pc_state::PcState,
    ports:      ports::Ports,
    interruptor: interruptor::Interruptor,
    raw_display: Vec<u8>,
}

impl<M: memory::MemoryRW> Core<M> {
    pub const IRQIM1ADDR: u16 = 0x38;

    pub fn new(clock: clocks::Clock, 
           memory: M, 
           pc_state: pc_state::PcState, 
           ports: ports::Ports,
           interruptor: interruptor::Interruptor) -> Self where M: memory::MemoryRW 
    {
    
        Self {
            clock: clock,
            memory: memory,
            pc_state: pc_state,
            ports: ports,
            interruptor: interruptor,
            raw_display: vec![0;(graphics::vdp::Constants::SMS_WIDTH as usize) * (graphics::vdp::Constants::SMS_HEIGHT as usize) * (graphics::vdp::Constants::BYTES_PER_PIXEL as usize)],
        }
    }

    fn interupt(&mut self) -> () {
        if self.pc_state.get_iff1() {
            if self.pc_state.get_im() == 1 {
                self.pc_state.increment_sp(-1);
                self.memory.write(self.pc_state.get_sp(), self.pc_state.get_pc_high());
                self.pc_state.increment_sp(-1);
                self.memory.write(self.pc_state.get_sp(), self.pc_state.get_pc_low());
                self.pc_state.set_pc(Core::<M>::IRQIM1ADDR);

                // Disable mask-able interrupts
                self.pc_state.set_iff1(false);
            } else {
                // TODO: Fix error messages/handling.
                println!("interupt mode not supported");
            }
        }
    }

    pub fn step(&mut self, debug: bool) -> (){
        // Start with 'expanded' version of step

        self.interruptor.set_cycle(self.clock.cycles);

        let op_code = self.memory.read(self.pc_state.get_pc());

        if debug {
            print!("{} {:x} {:x} ({:x} {:x}) ", self.clock.cycles, op_code, self.pc_state.get_pc(), op_code, self.memory.read(self.pc_state.get_pc() + 1));
            println!("{}", self.pc_state);
        }
        instructions::Instruction::execute(op_code, &mut self.clock, &mut self.memory, &mut self.pc_state, &mut self.ports, &mut self.interruptor);
        if self.ports.poll_interrupts(&mut self.raw_display, &self.clock) {
            self.interupt();
        }

    }

    pub fn generate_display(&mut self, buffer: &mut [u8], pitch: usize) -> () {
        // Function to populate the display buffer drawn to the 2D texture/canvas/window.
        buffer.clone_from_slice(self.raw_display.as_slice());
    }

}

#[test]
fn test_core_creation() {
    use super::super::graphics::vdp;

    let clock = clocks::Clock::new();
    let memory = memory::MemoryAbsolute::new();
    let pc_state = pc_state::PcState::new();
    let vdp = vdp::VDP::new();
    let mut ports = ports::Ports::new();
    let interruptor = interruptor::Interruptor::new();
    ports.add_device(Box::new(vdp));
    let mut core = Core::new(clock, memory, pc_state, ports, interruptor);

    core.step(true);
    println!("{}", core.pc_state);
    core.step(true);
}

