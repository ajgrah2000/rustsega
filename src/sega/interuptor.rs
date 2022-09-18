
pub struct Interuptor {
    pub next_interupt: u32
}

impl Interuptor {
    pub fn new () -> Self {
        Self {
            next_interupt: 0,
        }
    }
    // TODO: Add the actual interruptor trait/implementation (previously VDU).
    pub fn set_cycle(&mut self, cycles: u32) -> () {
        // TODO: Do something

    }
}
