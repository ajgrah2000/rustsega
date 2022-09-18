use bitfield::bitfield;
use std::fmt;

bitfield! {
    pub struct PcStatusFlagFields(u8);

    pub get_c,  set_c:  0,0;
    pub get_n,  set_n:  1,1;
    pub get_pv, set_pv: 2,2;
    pub get_x1, set_x1: 3,3;
    pub get_h,  set_h:  4,4;
    pub get_x2, set_x2: 5,5;
    pub get_z,  set_z:  6,6;
    pub get_s,  set_s:  7,7;
}

impl fmt::Display for PcStatusFlagFields {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        write!(dest, "(C:{} N:{} PV:{} X1:{} H:{} X2:{} Z:{} S:{})",
                self.get_c(), self.get_n(),  self.get_pv(), self.get_x1(), 
                self.get_h(), self.get_x2(), self.get_z(), self.get_s())
    }
}

pub struct Reg16 {
    high: u8,
    low: u8,
}

pub struct PcState {

        // Register overlays
        bc_reg: Reg16,
        de_reg: Reg16,
        af_reg: Reg16,
        hl_reg: Reg16,

        pc_reg: Reg16,
        sp_reg: Reg16,
        ix_reg: Reg16,
        iy_reg: Reg16,

        // Shadow registers
        bc__reg: Reg16,
        de__reg: Reg16,
        hl__reg: Reg16,
        af__reg: Reg16,

        r: u8, // TODO: Check, not sure if this is a 'real' register, used for random?
        iff1: u8,
        iff2: u8,
        im: u8,
}

impl fmt::Display for PcState {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let flags = self.get_f();
        write!(dest, "A:{:x} SP:{:x} B:{:x} C:{:x} D:{:x} E:{:x} H:{:x} L:{:x} F:{:x} PCHigh:{:x} PCLow:{:x} SPHigh:{:x} SPLow:{:x} IXHigh:{:x} IXLow:{:x} IYHigh:{:x} IYLow:{:x} {}", self.get_a(), self.get_sp(), self.get_b(), self.get_c(),self.get_d(),self.get_e(),self.get_h(),self.get_l(),self.get_f().0,self.get_pc_high(),self.get_pc_low(),self.get_sp_high(),self.get_sp_low(),self.get_ix_high(),self.get_ix_low(),self.get_iy_high(),self.get_iy_low(), self.get_f())
    }

}

impl Reg16 {
    fn new() -> Self {
        Self { 
            low:  0,
            high: 0,
        }
    }
    // Registers are stored 'big endian' as far as letter order, such that (b=1 as u8, c=0 as u8) -> (0x0100 as u16)
    fn set(&mut self, input: &u16) -> () {
        self.low  = (input & 0xFF) as u8;
        self.high = ((input >> 8) & 0xFF) as u8;
    }
    
    fn get(&self) -> u16 {
        let result = self.low as u16 + ((self.high as u16) << 8);
        result
    }
}

impl PcState {
    pub fn new() -> Self {
        Self {
            // Register overlays
            bc_reg: Reg16::new(),
            de_reg: Reg16::new(),
            af_reg: Reg16::new(), // 'F' is status flags
            hl_reg: Reg16::new(),

            pc_reg: Reg16::new(),
            sp_reg: Reg16::new(),
            ix_reg: Reg16::new(),
            iy_reg: Reg16::new(),

            // Shadow registers
            bc__reg: Reg16::new(),
            de__reg: Reg16::new(),
            hl__reg: Reg16::new(),
            af__reg: Reg16::new(),

            r: 0, // TODO: Check, not sure if this is a 'real' register, used for random?
            iff1: 0,
            iff2: 0,
            im: 0,
        }
    }

    pub fn get_b(&self) -> u8 {self.bc_reg.high}
    pub fn get_c(&self) -> u8 {self.bc_reg.low}
    pub fn get_d(&self) -> u8 {self.de_reg.high}
    pub fn get_e(&self) -> u8 {self.de_reg.low}
    pub fn get_a(&self) -> u8 {self.af_reg.high}
    pub fn get_f(&self) -> PcStatusFlagFields {PcStatusFlagFields(self.af_reg.low)}
    pub fn get_h(&self) -> u8 {self.hl_reg.high}
    pub fn get_l(&self) -> u8 {self.hl_reg.low}

    pub fn get_bc(&self) -> u16 {self.bc_reg.get()}
    pub fn get_de(&self) -> u16 {self.de_reg.get()}
    pub fn get_af(&self) -> u16 {self.af_reg.get()}
    pub fn get_hl(&self) -> u16 {self.hl_reg.get()}

    pub fn get_pc_high(&self) -> u8 {self.pc_reg.high}
    pub fn get_pc_low (&self) -> u8 {self.pc_reg.low}
    pub fn get_sp_high(&self) -> u8 {self.sp_reg.high}
    pub fn get_sp_low (&self) -> u8 {self.sp_reg.low}
    pub fn get_ix_high(&self) -> u8 {self.ix_reg.high}
    pub fn get_ix_low (&self) -> u8 {self.ix_reg.low}
    pub fn get_iy_high(&self) -> u8 {self.iy_reg.high}
    pub fn get_iy_low (&self) -> u8 {self.iy_reg.low}

    pub fn get_pc(&self) -> u16 {self.pc_reg.get()}
    pub fn get_sp(&self) -> u16 {self.sp_reg.get()}
    pub fn get_ix(&self) -> u16 {self.ix_reg.get()}
    pub fn get_iy(&self) -> u16 {self.iy_reg.get()}

    pub fn get_r   (&self) -> u8 {self.r}
    pub fn get_iff1(&self) -> u8 {self.iff1}
    pub fn get_iff2(&self) -> u8 {self.iff2}
    pub fn get_im  (&self) -> u8 {self.im}

    pub fn set_b(&mut self, input: &u8) -> () {self.bc_reg.high = *input;}
    pub fn set_c(&mut self, input: &u8) -> () {self.bc_reg.low  = *input;}
    pub fn set_d(&mut self, input: &u8) -> () {self.de_reg.high = *input;}
    pub fn set_e(&mut self, input: &u8) -> () {self.de_reg.low  = *input;}
    pub fn set_a(&mut self, input: &u8) -> () {self.af_reg.high = *input;}
    // TODO: Improve setting of flags, this probably won't be ideal, but it
    // will ensure 'af' and 'f' are always in sync.
    pub fn set_f(&mut self, input: PcStatusFlagFields) -> () {self.af_reg.low  = input.0;}
    pub fn set_h(&mut self, input: &u8) -> () {self.hl_reg.high = *input;}
    pub fn set_l(&mut self, input: &u8) -> () {self.hl_reg.low  = *input;}

    pub fn set_bc(&mut self, input: &u16) -> () {self.bc_reg.set(input);}
    pub fn set_de(&mut self, input: &u16) -> () {self.de_reg.set(input);}
    pub fn set_af(&mut self, input: &u16) -> () {self.af_reg.set(input);}
    pub fn set_hl(&mut self, input: &u16) -> () {self.hl_reg.set(input);}

    pub fn set_pc_high(&mut self, input: &u8) -> () {self.pc_reg.high = *input;}
    pub fn set_pc_low (&mut self, input: &u8) -> () {self.pc_reg.low  = *input;}
    pub fn set_sp_high(&mut self, input: &u8) -> () {self.sp_reg.high = *input;}
    pub fn set_sp_low (&mut self, input: &u8) -> () {self.sp_reg.low  = *input;}
    pub fn set_ix_high(&mut self, input: &u8) -> () {self.ix_reg.high = *input;}
    pub fn set_ix_low (&mut self, input: &u8) -> () {self.ix_reg.low  = *input;}
    pub fn set_iy_high(&mut self, input: &u8) -> () {self.iy_reg.high = *input;}
    pub fn set_iy_low (&mut self, input: &u8) -> () {self.iy_reg.low  = *input;}

    pub fn set_pc(&mut self, input: &u16) -> () {self.pc_reg.set(input);}
    pub fn set_sp(&mut self, input: &u16) -> () {self.sp_reg.set(input);}
    pub fn set_ix(&mut self, input: &u16) -> () {self.ix_reg.set(input);}
    pub fn set_iy(&mut self, input: &u16) -> () {self.iy_reg.set(input);}

    pub fn set_r   (&mut self, input: u8) -> () {self.r    = input;}
    pub fn set_iff1(&mut self, input: u8) -> () {self.iff1 = input;}
    pub fn set_iff2(&mut self, input: u8) -> () {self.iff2 = input;}
    pub fn set_im  (&mut self, input: u8) -> () {self.im   = input;}

    // Additional utility functions, intended to simplify some of the calls.
    pub fn increment_reg(register: &mut Reg16, increment: i8) -> () {
        let update_value = ((register.get() as i16) + (increment as i16)) as u16;
        register.set(&update_value);
    }

    pub fn increment_sp(&mut self, increment: i8) -> () {Self::increment_reg(&mut self.sp_reg, increment);}
    pub fn increment_pc(&mut self, increment: i8) -> () {Self::increment_reg(&mut self.pc_reg, increment);}
}


#[test]
fn test_pc_status_flag_fields() {
    let mut pc_status_flags = PcStatusFlagFields(0);
    pc_status_flags.set_c(1);
    assert_eq!(pc_status_flags.get_c(),1);
    pc_status_flags.set_n(1);
    assert_eq!(pc_status_flags.0,3);
}

#[test]
fn test_pc_state_16_changes() {
    let mut pc_state = PcState::new();
    let mut value_8:u8 = 7;
    let mut value_16:u16 = 0x312;

    pc_state.set_b(&value_8);
    assert_eq!(pc_state.get_b(),  7);
    assert_eq!(pc_state.get_c(),  0);
    assert_eq!(pc_state.get_bc(), 0x700);

    pc_state.set_bc(&value_16);
    assert_eq!(pc_state.get_bc(), 0x312);
    assert_eq!(pc_state.get_b(),  0x3);
    assert_eq!(pc_state.get_c(),  0x12);

    assert_eq!(pc_state.get_af(), 0x0);
    pc_state.set_af(&value_16);
    assert_eq!(pc_state.get_a(),  0x03);
    assert_eq!(pc_state.get_f().get_c(),  0x0);
    assert_eq!(pc_state.get_f().get_n(),  0x1);
    assert_eq!(pc_state.get_f().get_pv(), 0x0);
    assert_eq!(pc_state.get_f().get_x1(), 0x0);
    assert_eq!(pc_state.get_f().get_h(),  0x1);
    assert_eq!(pc_state.get_f().get_x2(), 0x0);
    assert_eq!(pc_state.get_f().get_z(),  0x0);
    assert_eq!(pc_state.get_f().get_s(),  0x0);

    value_16 = 0x80FE;
    pc_state.set_af(&value_16);
    assert_eq!(pc_state.get_a(),  0x80);
    assert_eq!(pc_state.get_f().get_c(),  0x0);
    assert_eq!(pc_state.get_f().get_n(),  0x1);
    assert_eq!(pc_state.get_f().get_pv(), 0x1);
    assert_eq!(pc_state.get_f().get_x1(), 0x1);
    assert_eq!(pc_state.get_f().get_h(),  0x1);
    assert_eq!(pc_state.get_f().get_x2(), 0x1);
    assert_eq!(pc_state.get_f().get_z(),  0x1);
    assert_eq!(pc_state.get_f().get_s(),  0x1);
    assert_eq!(format!("{}", pc_state), "A:80 SP:0 B:3 C:12 D:0 E:0 H:0 L:0 F:fe PCHigh:0 PCLow:0 SPHigh:0 SPLow:0 IXHigh:0 IXLow:0 IYHigh:0 IYLow:0 (C:0 N:1 PV:1 X1:1 H:1 X2:1 Z:1 S:1)");

    value_16 = 0xdffd;
    pc_state.set_sp(&value_16);
    let mut flags = pc_state.get_f();
    flags.set_x1(0);
    pc_state.set_f(flags);
    // Use the formatted state to check the output.
    assert_eq!(format!("{}", pc_state), "A:80 SP:dffd B:3 C:12 D:0 E:0 H:0 L:0 F:f6 PCHigh:0 PCLow:0 SPHigh:df SPLow:fd IXHigh:0 IXLow:0 IYHigh:0 IYLow:0 (C:0 N:1 PV:1 X1:0 H:1 X2:1 Z:1 S:1)")
}
