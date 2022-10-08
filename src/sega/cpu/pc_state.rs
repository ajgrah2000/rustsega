use bitfield::bitfield;
use std::fmt;

bitfield! {
    pub struct PcStatusFlagFields(u8);

    pub get_c,  set_c:  0,0;

    // N - Add/Subtract flag. Cleared for ADD instructions, set to 1 for SUB
    pub get_n,  set_n:  1,1;

    // Overflow flag (assuming signed arguments, check for overflow addition/subtractions)
    pub get_pv, set_pv: 2,2;
    pub get_x1, set_x1: 3,3;
    pub get_h,  set_h:  4,4;
    pub get_x2, set_x2: 5,5;

    // Zero flag.  set to 1 if the result generated by the execution of certain
    // instructions is 0.
    pub get_z,  set_z:  6,6;

    // Sign flag, most significant bit of the accumulator.
    // Changes when inputting a byte from I/O to a register using IN r,(C)
    pub get_s,  set_s:  7,7;
}

impl fmt::Display for PcStatusFlagFields {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        write!(
            dest,
            "(C:{} N:{} PV:{} X1:{} H:{} X2:{} Z:{} S:{})",
            self.get_c(),
            self.get_n(),
            self.get_pv(),
            self.get_x1(),
            self.get_h(),
            self.get_x2(),
            self.get_z(),
            self.get_s()
        )
    }
}

pub struct Reg16 {
    pub high: u8,
    pub low: u8,
}

pub struct FlagReg16 {
    // TODO: I'm sure there's a better way to handle the flag overlay with
    // 'af'.  This works, can revisit later.
    reg16: Reg16,
}

pub struct PcState {
    // Register overlays
    pub bc_reg: Reg16,
    pub de_reg: Reg16,
    pub af_reg: FlagReg16,
    pub hl_reg: Reg16,

    pub pc_reg: Reg16,
    pub sp_reg: Reg16,
    pub ix_reg: Reg16,
    pub iy_reg: Reg16,

    // Shadow registers
    pub shadow_bc_reg: Reg16,
    pub shadow_de_reg: Reg16,
    pub shadow_hl_reg: Reg16,
    pub shadow_af_reg: FlagReg16,

    r: u8, // Memory refresh register, lower 7 bits increment after each instruction fetch. 8-th bit only set by LD R,A
    i: u8, // Interrupt register
    iff1: bool,
    iff2: bool,
    im: u8,
}

impl fmt::Display for PcState {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        write!(dest, "A:{:x} SP:{:x} B:{:x} C:{:x} D:{:x} E:{:x} H:{:x} L:{:x} F:{:x} PCHigh:{:x} PCLow:{:x} SPHigh:{:x} SPLow:{:x} IXHigh:{:x} IXLow:{:x} IYHigh:{:x} IYLow:{:x} {}", self.get_a(), self.get_sp(), self.get_b(), self.get_c(),self.get_d(),self.get_e(),self.get_h(),self.get_l(),self.get_f().0,self.get_pc_high(),self.get_pc_low(),self.get_sp_high(),self.get_sp_low(),self.get_ix_high(),self.get_ix_low(),self.get_iy_high(),self.get_iy_low(), self.get_f())
    }
}

impl Reg16 {
    fn new() -> Self {
        Self { low: 0, high: 0 }
    }
    // Registers are stored 'big endian' as far as letter order, such that (b=1 as u8, c=0 as u8) -> (0x0100 as u16)
    pub fn set(&mut self, input: u16) {
        self.low = (input & 0xFF) as u8;
        self.high = ((input >> 8) & 0xFF) as u8;
    }

    pub fn get(&self) -> u16 {
        self.low as u16 + ((self.high as u16) << 8)
    }
}

pub trait Reg16RW {
    fn set(&mut self, input: u16);
    fn get(&self) -> u16;
    // Used by stack pointer
    fn set_low(&mut self, input: u8);
    fn set_high(&mut self, input: u8);
    fn get_low(&self) -> u8;
    fn get_high(&self) -> u8;
}

pub trait FlagReg {
    fn get_flags(&self) -> PcStatusFlagFields;
    fn set_flags(&mut self, flags: &PcStatusFlagFields);
}

pub trait AfRegister {
    fn get_a(&self) -> u8;
    fn get_f(&self) -> u8;
}

impl Reg16RW for Reg16 {
    fn get(&self) -> u16 {
        self.get()
    }

    fn set(&mut self, input: u16) {
        self.set(input)
    }
    fn set_low(&mut self, input: u8) {
        self.low = input;
    }
    fn set_high(&mut self, input: u8) {
        self.high = input;
    }

    fn get_low(&self) -> u8 {
        self.low
    }
    fn get_high(&self) -> u8 {
        self.high
    }
}

impl FlagReg16 {
    fn new() -> Self {
        Self {
            reg16: Reg16::new(),
        }
    }
}

impl FlagReg for FlagReg16 {
    fn get_flags(&self) -> PcStatusFlagFields {
        PcStatusFlagFields(self.reg16.low)
    }

    fn set_flags(&mut self, flags: &PcStatusFlagFields) {
        self.reg16.low = flags.0;
    }
}

impl AfRegister for FlagReg16 {
    fn get_a(&self) -> u8 {
        self.reg16.high
    }

    fn get_f(&self) -> u8 {
        self.reg16.low
    }
}

impl Reg16RW for FlagReg16 {
    fn set(&mut self, input: u16) {
        self.reg16.set(input)
    }

    fn get(&self) -> u16 {
        self.reg16.get()
    }

    fn set_low(&mut self, input: u8) {
        self.reg16.low = input;
    }
    fn set_high(&mut self, input: u8) {
        self.reg16.high = input;
    }
    fn get_low(&self) -> u8 {
        self.reg16.low
    }
    fn get_high(&self) -> u8 {
        self.reg16.high
    }
}

impl PcState {
    pub fn new() -> Self {
        Self {
            // Register overlays
            bc_reg: Reg16::new(),
            de_reg: Reg16::new(),
            af_reg: FlagReg16::new(), // 'F' is status flags
            hl_reg: Reg16::new(),

            pc_reg: Reg16::new(),
            sp_reg: Reg16::new(),
            ix_reg: Reg16::new(),
            iy_reg: Reg16::new(),

            // Shadow registers
            shadow_bc_reg: Reg16::new(),
            shadow_de_reg: Reg16::new(),
            shadow_hl_reg: Reg16::new(),
            shadow_af_reg: FlagReg16::new(),

            r: 0,
            i: 0,
            iff1: false,
            iff2: false,
            im: 0,
        }
    }

    pub fn get_b(&self) -> u8 {
        self.bc_reg.high
    }
    pub fn get_c(&self) -> u8 {
        self.bc_reg.low
    }
    pub fn get_d(&self) -> u8 {
        self.de_reg.high
    }
    pub fn get_e(&self) -> u8 {
        self.de_reg.low
    }
    pub fn get_a(&self) -> u8 {
        self.af_reg.reg16.high
    }
    pub fn get_f(&self) -> PcStatusFlagFields {
        self.af_reg.get_flags()
    }
    pub fn get_h(&self) -> u8 {
        self.hl_reg.high
    }
    pub fn get_l(&self) -> u8 {
        self.hl_reg.low
    }

    pub fn get_bc(&self) -> u16 {
        self.bc_reg.get()
    }
    pub fn get_de(&self) -> u16 {
        self.de_reg.get()
    }
    pub fn get_af(&self) -> u16 {
        self.af_reg.reg16.get()
    }
    pub fn get_hl(&self) -> u16 {
        self.hl_reg.get()
    }

    pub fn get_pc_high(&self) -> u8 {
        self.pc_reg.high
    }
    pub fn get_pc_low(&self) -> u8 {
        self.pc_reg.low
    }
    pub fn get_sp_high(&self) -> u8 {
        self.sp_reg.high
    }
    pub fn get_sp_low(&self) -> u8 {
        self.sp_reg.low
    }
    pub fn get_ix_high(&self) -> u8 {
        self.ix_reg.high
    }
    pub fn get_ix_low(&self) -> u8 {
        self.ix_reg.low
    }
    pub fn get_iy_high(&self) -> u8 {
        self.iy_reg.high
    }
    pub fn get_iy_low(&self) -> u8 {
        self.iy_reg.low
    }

    pub fn get_pc(&self) -> u16 {
        self.pc_reg.get()
    }
    pub fn get_sp(&self) -> u16 {
        self.sp_reg.get()
    }

    pub fn get_r(&self) -> u8 {
        self.r
    }
    pub fn get_i(&self) -> u8 {
        self.i
    }
    pub fn get_iff1(&self) -> bool {
        self.iff1
    }
    pub fn get_iff2(&self) -> bool {
        self.iff2
    }
    pub fn get_im(&self) -> u8 {
        self.im
    }

    pub fn set_b(&mut self, input: u8) {
        self.bc_reg.high = input;
    }
    pub fn set_c(&mut self, input: u8) {
        self.bc_reg.low = input;
    }
    pub fn set_d(&mut self, input: u8) {
        self.de_reg.high = input;
    }
    pub fn set_e(&mut self, input: u8) {
        self.de_reg.low = input;
    }
    pub fn set_a(&mut self, input: u8) {
        self.af_reg.reg16.high = input;
    }
    // TODO: Improve setting of flags, this probably won't be ideal, but it
    // will ensure 'af' and 'f' are always in sync.
    pub fn set_f(&mut self, input: PcStatusFlagFields) {
        self.af_reg.set_flags(&input);
    }
    pub fn set_h(&mut self, input: u8) {
        self.hl_reg.high = input;
    }
    pub fn set_l(&mut self, input: u8) {
        self.hl_reg.low = input;
    }

    pub fn set_bc(&mut self, input: u16) {
        self.bc_reg.set(input);
    }
    pub fn set_de(&mut self, input: u16) {
        self.de_reg.set(input);
    }
    pub fn set_af(&mut self, input: u16) {
        self.af_reg.reg16.set(input);
    }
    pub fn set_hl(&mut self, input: u16) {
        self.hl_reg.set(input);
    }

    pub fn set_pc_high(&mut self, input: u8) {
        self.pc_reg.high = input;
    }
    pub fn set_pc_low(&mut self, input: u8) {
        self.pc_reg.low = input;
    }

    pub fn set_pc(&mut self, input: u16) {
        self.pc_reg.set(input);
    }

    pub fn set_r(&mut self, input: u8) {
        self.r = input;
    }
    pub fn set_i(&mut self, input: u8) {
        self.i = input;
    }
    pub fn set_iff1(&mut self, input: bool) {
        self.iff1 = input;
    }
    pub fn set_iff2(&mut self, input: bool) {
        self.iff2 = input;
    }
    pub fn set_im(&mut self, input: u8) {
        self.im = input;
    }

    // Additional utility functions, intended to simplify some of the calls.
    pub fn increment_reg(register: &mut dyn Reg16RW, increment: i8) {
        let update_value = ((register.get() as i16) + (increment as i16)) as u16;
        register.set(update_value);
    }

    pub fn increment_sp(&mut self, increment: i8) {
        Self::increment_reg(&mut self.sp_reg, increment);
    }
    pub fn increment_pc(&mut self, increment: i8) {
        Self::increment_reg(&mut self.pc_reg, increment);
    }
}

#[test]
fn test_pc_status_flag_fields() {
    let mut pc_status_flags = PcStatusFlagFields(0);
    pc_status_flags.set_c(1);
    assert_eq!(pc_status_flags.get_c(), 1);
    pc_status_flags.set_n(1);
    assert_eq!(pc_status_flags.0, 3);
}

#[test]
fn test_pc_state_16_changes() {
    let mut pc_state = PcState::new();
    let value_8: u8 = 7;
    let mut value_16: u16 = 0x312;

    pc_state.set_b(value_8);
    assert_eq!(pc_state.get_b(), 7);
    assert_eq!(pc_state.get_c(), 0);
    assert_eq!(pc_state.get_bc(), 0x700);

    pc_state.set_bc(value_16);
    assert_eq!(pc_state.get_bc(), 0x312);
    assert_eq!(pc_state.get_b(), 0x3);
    assert_eq!(pc_state.get_c(), 0x12);

    assert_eq!(pc_state.get_af(), 0x0);
    pc_state.set_af(value_16);
    assert_eq!(pc_state.get_a(), 0x03);
    assert_eq!(pc_state.get_f().get_c(), 0x0);
    assert_eq!(pc_state.get_f().get_n(), 0x1);
    assert_eq!(pc_state.get_f().get_pv(), 0x0);
    assert_eq!(pc_state.get_f().get_x1(), 0x0);
    assert_eq!(pc_state.get_f().get_h(), 0x1);
    assert_eq!(pc_state.get_f().get_x2(), 0x0);
    assert_eq!(pc_state.get_f().get_z(), 0x0);
    assert_eq!(pc_state.get_f().get_s(), 0x0);

    value_16 = 0x80FE;
    pc_state.set_af(value_16);
    assert_eq!(pc_state.get_a(), 0x80);
    assert_eq!(pc_state.get_f().get_c(), 0x0);
    assert_eq!(pc_state.get_f().get_n(), 0x1);
    assert_eq!(pc_state.get_f().get_pv(), 0x1);
    assert_eq!(pc_state.get_f().get_x1(), 0x1);
    assert_eq!(pc_state.get_f().get_h(), 0x1);
    assert_eq!(pc_state.get_f().get_x2(), 0x1);
    assert_eq!(pc_state.get_f().get_z(), 0x1);
    assert_eq!(pc_state.get_f().get_s(), 0x1);
    assert_eq!(format!("{}", pc_state), "A:80 SP:0 B:3 C:12 D:0 E:0 H:0 L:0 F:fe PCHigh:0 PCLow:0 SPHigh:0 SPLow:0 IXHigh:0 IXLow:0 IYHigh:0 IYLow:0 (C:0 N:1 PV:1 X1:1 H:1 X2:1 Z:1 S:1)");

    value_16 = 0xdffd;
    pc_state.sp_reg.set(value_16);
    let mut flags = pc_state.get_f();
    flags.set_x1(0);
    pc_state.set_f(flags);
    // Use the formatted state to check the output.
    assert_eq!(format!("{}", pc_state), "A:80 SP:dffd B:3 C:12 D:0 E:0 H:0 L:0 F:f6 PCHigh:0 PCLow:0 SPHigh:df SPLow:fd IXHigh:0 IXLow:0 IYHigh:0 IYLow:0 (C:0 N:1 PV:1 X1:0 H:1 X2:1 Z:1 S:1)")
}
