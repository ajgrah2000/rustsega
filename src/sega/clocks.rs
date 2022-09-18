
pub struct Clock {
    pub cycles: u32
}

impl Clock {
    pub fn new () -> Self {
        Self {
            cycles: 0,
        }
    }

    pub fn increment(&mut self, inc: u32) -> () {
        self.cycles = self.cycles + inc;
    }
}
