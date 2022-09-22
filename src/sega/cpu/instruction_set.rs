use super::pc_state;
use super::super::memory::memory;
use super::super::ports;
use super::super::clocks;
use super::flagtables;

//0x00
pub fn noop(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    pc_state.increment_pc(1);
    clock.increment(4);
}

//0xDB 
// IN self.pc_state.A, (N)
pub fn in_a_n(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
              pc_state: &mut pc_state::PcState, ports: &mut ports::Ports) -> () {

    pc_state.set_a(ports.port_read(memory.read(pc_state.get_pc() + 1)));
    pc_state.increment_pc(2);
    clock.increment(11);
}

//0xF3, disable interrupts
pub fn di(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    pc_state.set_iff1(0);
    pc_state.set_iff2(0);
    pc_state.increment_pc(1);
    clock.increment(4);
}

// # self.pc_state.IM 1
pub fn im_1(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    pc_state.increment_pc(2);
    pc_state.set_im(1);

    clock.increment(2);
}

pub fn signed_char_to_int(v: i8) -> i16 {
    return v as i16;
}

// self.pc_state.Add two 8 bit ints plus the carry bit, and set flags accordingly
fn u8_carry(pc_state: &mut pc_state::PcState, a:u8, b:u8, c:bool) -> u8 {
    let mut f_status = pc_state.get_f();
    let r = a.wrapping_add(b).wrapping_add(u8::from(c));

    if (r & 0x80) != 0 { // Negative
        f_status.set_s(1);
    } else {
        f_status.set_s(0);
    }
 
    if (r & 0xFF) == 0 { // Zero
        f_status.set_z(1);
    } else {
        f_status.set_z(0);
    }
 
    // An Overflow can't occur if a and b have different sign bits
    // If they're the same, an overflow occurred if the sign of the result changed.
    // Basically, tread both arguments as signed numbers

    if (((a & 0x80) ^ (b & 0x80)) == 0x00) && // arguments same sign
       (((a & 0x80) ^ (r & 0x80)) == 0x80) {  // result different sign
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }

    f_status.set_h(0);
    if (((a & 0xF) + (b & 0xF) + u8::from(c)) & 0x10) == 0x10 { // Half carry
        f_status.set_h(1);
    } else {
        f_status.set_h(0);
    }
 
    if c {
        if a > 0xFF - b {
            f_status.set_c(1);
        } else {
            f_status.set_c(0);
        }
    } else {
        if a >= 0xFF - b {
            f_status.set_c(1);
        } else {
            f_status.set_c(0);
        }
    }
    
    pc_state.set_f(f_status);
 
    return r;
}

fn add8(pc_state: &mut pc_state::PcState, a:u8, b:u8) -> u8 {
    // Just call the add c function.
    add8c(pc_state, a, b, false)
}

fn add8c(pc_state: &mut pc_state::PcState, a:u8, b:u8, c:bool) -> u8 {
    let mut f_status = pc_state.get_f();
    f_status.set_n(0); // Clear N to indicate add
    pc_state.set_f(f_status);
    u8_carry(pc_state, a, b, c) 
}

fn cp_flags(pc_state: &mut pc_state::PcState, a:u8, b:u8) -> () {
    // CP flags calculated set the same as for subtaction, but the result is ignored.
    sub8c(pc_state, a, b, false);
}

// Subtract two 8 bit ints and the carry bit, set flags accordingly
fn sub8c(pc_state: &mut pc_state::PcState, a:u8, b:u8, c:bool) -> u8 {
    let mut f_status = pc_state.get_f();
    f_status.set_n(1); // Set N to indicate subtract
    pc_state.set_f(f_status);

    // a - b + c -> a + (~b + 1) + c -> a + ~b - c
    u8_carry(pc_state, a, !b, !c)
}

fn u16_carry(pc_state: &mut pc_state::PcState, a:u16, b:u16, c:bool) -> u16 {
    // Perform a u16-bit add with carry, setting the flags (except N, which is
    // left to add/sub)
    let mut f_status = pc_state.get_f();
    let r = a.wrapping_add(b).wrapping_add(u16::from(c));

    if (r & 0x8000) != 0 { // Negative
        f_status.set_s(1);
    } else {
        f_status.set_s(0);
    }
 
    if (r & 0xFFFF) == 0 { // Zero
        f_status.set_z(1);
    } else {
        f_status.set_z(0);
    }
 
    // An Overflow can't occur if a and b have different sign bits
    // If they're the same, an overflow occurred if the sign of the result changed.
    // Basically, tread both arguments as signed numbers

    if (((a & 0x8000) ^ (b & 0x8000)) == 0x0000) && // arguments same sign
       (((a & 0x8000) ^ (r & 0x8000)) == 0x8000) {  // result different sign
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }

    f_status.set_h(0);
    if (((a & 0xFFF) + (b & 0xFFF) + u16::from(c)) & 0x1000) == 0x1000 { // Half carry
        f_status.set_h(1);
    } else {
        f_status.set_h(0);
    }
 
    if c {
        if a > 0xFFFF - b {
            f_status.set_c(1);
        } else {
            f_status.set_c(0);
        }
    } else {
        if a >= 0xFFFF - b {
            f_status.set_c(1);
        } else {
            f_status.set_c(0);
        }
    }
    
    pc_state.set_f(f_status);
 
    return r;
}

fn add16c(pc_state: &mut pc_state::PcState, a:u16, b:u16, c:bool) -> u16 {
    let mut f_status = pc_state.get_f();
    f_status.set_n(0);
    pc_state.set_f(f_status);
    u16_carry(pc_state, a, b, c)
}

fn sub16c(pc_state: &mut pc_state::PcState, a:u16, b:u16, c:bool) -> u16 {
    let mut f_status = pc_state.get_f();
    f_status.set_n(1);
    pc_state.set_f(f_status);
    // a - b + c -> a + (~b + 1) + c -> a + ~b - c
    u16_carry(pc_state, a, !b, !c)
}

// Calculate the result of the DAA functio
fn calculate_daa_add(pc_state: &mut pc_state::PcState) -> () {
    let upper = (pc_state.get_a() >> 4) & 0xF;
    let lower =  pc_state.get_a() & 0xF;
    
    let mut f_status = pc_state.get_f();

    if f_status.get_c() == 0 {
        if (upper <= 9) && (f_status.get_h() == 0) && (lower <= 9) {
            // Do nothing
        } else if (upper <= 8) && (f_status.get_h() == 0) && ((lower >= 0xA) && (lower <= 0xF)) {
            pc_state.set_a(pc_state.get_a() + 0x06);
        } else if (upper <= 9) && (f_status.get_h() == 1) && (lower <= 0x3) {
            pc_state.set_a(pc_state.get_a() + 0x06);
        } else if ((upper >= 0xA) && (upper <= 0xF)) && (f_status.get_h() == 0) && (lower <= 0x9) {
            pc_state.set_a(pc_state.get_a() + 0x60);
            f_status.set_c(1);
        } else if ((upper >= 0x9) && (upper <= 0xF)) && (f_status.get_h() == 0) && ((lower >= 0xA) && (lower <= 0xF)) {
            pc_state.set_a(pc_state.get_a() + 0x66);
            f_status.set_c(1);
        } else if ((upper >= 0xA) && (upper <= 0xF)) && (f_status.get_h() == 1) && (lower <= 0x3) {
            pc_state.set_a(pc_state.get_a() + 0x66);
            f_status.set_c(1);
        }
    } else {
        if (upper <= 0x2) && (f_status.get_h() == 0) && (lower <= 0x9) {
            pc_state.set_a(pc_state.get_a() + 0x60);
        } else if (upper <= 0x2) && (f_status.get_h() == 0) && ((lower >= 0xA) && (lower <= 0xF)) {
            pc_state.set_a(pc_state.get_a() + 0x66);
        } else if (upper <= 0x3) && (f_status.get_h() == 1) && (lower <= 0x3) {
            pc_state.set_a(pc_state.get_a() + 0x66);
        }
    }

    f_status.set_pv(u8::from(flagtables::calculate_parity(pc_state.get_a())));
    if (pc_state.get_a() & 0x80) != 0 { // Is negative
        f_status.set_s(1);
    } else {
        f_status.set_s(0);
    }

    if pc_state.get_a() == 0 { // Is zero
        f_status.set_z(1);
    } else {
        f_status.set_z(0);
    }

    pc_state.set_f(f_status);
}

// Fcpu_state->IXME, table in z80 guide is wrong, need to check values by hand
fn calculate_daa_sub(pc_state: &mut pc_state::PcState) {
    let upper = (pc_state.get_a() >> 4) & 0xF;
    let lower =  pc_state.get_a() & 0xF;
    
    let mut f_status = pc_state.get_f();

    if f_status.get_c() == 0 {
        if (upper <= 9) && (f_status.get_h() == 0) && (lower <= 9) {
            // Do nothing
        } else if (upper <= 0x8) && (f_status.get_h() == 1) && ((lower >= 0x6) && (lower <= 0xF)) {
            pc_state.set_a(pc_state.get_a() + 0xFA);
        }
    } else {
        if ((upper >= 0x7) && (upper <= 0xF)) && (f_status.get_h() == 0) && (lower <= 0x9) {
            pc_state.set_a(pc_state.get_a() + 0xA0);
        } else if ((upper >= 0x6) && (upper <= 0xF)) && (f_status.get_h() == 1) && ((lower >= 0x6) && (lower <= 0xF)) {
            f_status.set_h(0);
            pc_state.set_a(pc_state.get_a() + 0x9A);
        }
    }

    f_status.set_pv(u8::from(flagtables::calculate_parity(pc_state.get_a())));
    if (pc_state.get_a() & 0x80) != 0 { // Is negative
        f_status.set_s(1);
    } else {
        f_status.set_s(0);
    }

    if pc_state.get_a() == 0 { // Is zero
        f_status.set_z(1);
    } else {
        f_status.set_z(0);
    }

    pc_state.set_f(f_status);
}

/*************************************************************************************/
/* Load Instructions                                                                 */
/*************************************************************************************/

//  LD dd, nn, Load a 16-bit register with the value 'nn'
pub fn ld_16_nn(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
              pc_reg: &mut pc_state::Reg16, r16_reg: &mut pc_state::Reg16) -> () {
    r16_reg.set(memory.read16(pc_reg.get() +1)); 

    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(10);
}

// LD (16 REG), r
// eg: LD (HL), r
// Load the 8-bit register, r, 16-bit address
pub fn ld_mem_r(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
                r: u8, pc_reg: &mut pc_state::Reg16, address_reg: &pc_state::Reg16) -> () {
    memory.write(address_reg.get(), r);
    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(7);
}

// LD r,r
pub fn ld_r_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, src: u8, pc_state: &mut pc_state::PcState, mut dst_fn: F) -> () {
    dst_fn(pc_state, src);
    pc_state.increment_pc(1);
    clock.increment(4);
}

// LD r, (16 REG)
// eg LD r, (HL)
pub fn ld_r_mem<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState, mut dst_fn: F, addr_reg_value: u16) -> () {
    dst_fn(pc_state, memory.read(addr_reg_value));
    pc_state.increment_pc(1);
    clock.increment(7);
}

// LD r,n
pub fn ld_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState, mut dst_fn: F) -> () {
    dst_fn(pc_state, memory.read(pc_state.get_pc() + 1));
    pc_state.increment_pc(2);
    clock.increment(7);
}

// LD r, (nn)
// Load the value from the 16-bit address into the 16-bit register
pub fn ld_r16_mem(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
              pc_reg: &mut pc_state::Reg16, r16_reg: &mut pc_state::Reg16) -> () {
    r16_reg.set(memory.read16(memory.read16(pc_reg.get()+1)));
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(20);
}

// LD (16 REG), n
// eg LD (HL), n
// Load the value 'n' into the 16-bit address
pub fn ld_mem_n(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
              pc_reg: &mut pc_state::Reg16, r16_reg: &mut pc_state::Reg16) -> () {
    // Load the 8 bit value 'n' into memory.
    memory.write(r16_reg.get(), memory.read(pc_reg.get() + 1));
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(10);
}

// LD r, (nn)
// eg LD A, (nn)  (is actually the only version)
// Op Code: 3A
// Load the value from the 16-bit address into the 8-bit register
pub fn ld_r8_mem<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState, mut dst_fn: F) -> () {
    dst_fn(pc_state, memory.read(memory.read16(pc_state.get_pc() +1 )));
    pc_state.increment_pc(3);
    clock.increment(13);
}

//  
//  LD SP, HL Load a 16-bit register with the value from another 16-bit register
pub fn ld_sp_hl(clock: &mut clocks::Clock, hl_reg: &pc_state::Reg16, 
                pc_reg: &mut pc_state::Reg16, sp_reg: &mut pc_state::Reg16) -> () {
    sp_reg.set(hl_reg.get()); 

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(6);
}

// LD (nn), r
// eg LD (nn), A   - Which is the only version of this function.
pub fn ld_nn_r(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, r: u8,
              pc_reg: &mut pc_state::Reg16) -> () {
    memory.write(memory.read16(pc_reg.get()+1), r);
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(13);
}

// LD (nn), HL
pub fn ld_nn_hl(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    memory.write(memory.read16(pc_state.get_pc()+2), pc_state.get_l());
    memory.write(memory.read16(pc_state.get_pc()+2)+1, pc_state.get_h());

    pc_state.increment_pc(4);
    clock.increment(16);
}

/*************************************************************************************/
/* Compare Instructions                                                              */
/*************************************************************************************/

// CP n
// Compare accumulator with 'n' to set status flags (but don't change accumulator)
pub fn cp_n(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state, pc_state.get_a(),  memory.read(pc_state.get_pc() +1));

    pc_state.increment_pc(2);
    clock.increment(7);
}

// CP r
// Compare accumulator with register r to set status flags (but don't change accumulator)
pub fn cp_r(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, r: u8, pc_state: &mut pc_state::PcState) -> () {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state, pc_state.get_a(),  r);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// CP (hl)
// Compare accumulator with the value from (HL) to set status flags (but don't change accumulator)
pub fn cp_hl(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state, pc_state.get_a(), memory.read(pc_state.get_hl()));

    pc_state.increment_pc(1);
    clock.increment(7);
}

/*************************************************************************************/
/* JUMP Instructions                                                                 */
/*************************************************************************************/

pub fn jp_nn(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
              pc_state: &mut pc_state::PcState) -> () {
    pc_state.set_pc(memory.read16(pc_state.get_pc() + 1));
    clock.increment(10);
}

//  JP (HL)
// Load PC with HL, to jump to that location.
pub fn jp_hl(clock: &mut clocks::Clock, hl_reg: &pc_state::Reg16, pc_reg: &mut pc_state::Reg16) -> () {
    pc_reg.set(hl_reg.get()); 
    clock.increment(4);
}

// Jump relative condition.  (instruction grouping isn't as convinient as for 'JP cc, nn')
// JR cc, e 
pub fn jr_cc_e(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState, condition: bool) -> () {
    clock.increment(7);

    if condition {
        pc_state.increment_pc(memory.read(pc_state.get_pc() + 1) as i8);
        clock.increment(5);
    }
    pc_state.increment_pc(2);
}

// JR NZ, e
pub fn jrnz_e(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_z() == 0);
}

// JR Z, e
pub fn jrz_e(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_z() == 1);
}

// JR NC, e
pub fn jrnc_e(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_c() == 0);
}

// JR C, e
pub fn jrc_e(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_c() == 1);
}

// Relative jump
// JR e
pub fn jr_e(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    // Timing for this is the same as for conditional jump relative.
    jr_cc_e(clock, memory, pc_state, true);
}

// Absolute Jump on condition
// JP cc, nn
pub fn jump_cc_nn(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState, condition:bool) -> () {
    if condition {
        pc_state.set_pc(memory.read16(pc_state.get_pc() + 1));
        clock.increment(5);
    } else {
        pc_state.increment_pc(3);
    }

    clock.increment(10);
}

// DEC r
// Decrement register and set status flags.
// Slowest dec function ever, why didn't Zilog come up with simpler instruction.
pub fn dec_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let new_value =  src.wrapping_sub(1);
    dst_fn(pc_state, new_value);

    let mut f_value = pc_state.get_f();
    flagtables::calculate_dec_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// DEC ss
pub fn dec_16<F: FnMut(&mut pc_state::PcState, u16)-> ()> (clock: &mut clocks::Clock, mut reg16: F, pc_state: &mut pc_state::PcState, original: u16) -> () {
    reg16(pc_state, original.wrapping_sub(1));
    pc_state.increment_pc(1);
    clock.increment(6);
}

// DEC (HL)
// Decrement (HL) and set status flags.
pub fn dec_hl(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {

    let new_value =  memory.read(pc_state.get_hl()).wrapping_sub(1);
    memory.write(pc_state.get_hl(), new_value);

    let mut f_value = pc_state.get_f();
    flagtables::calculate_dec_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(11);
}

// INC r
// Increment register and set status flags.
pub fn inc_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let new_value =  src.wrapping_add(1);
    dst_fn(pc_state, new_value);

    let mut f_value = pc_state.get_f();
    flagtables::calculate_inc_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// INC ss
pub fn inc_16<F: FnMut(&mut pc_state::PcState, u16)-> ()> (clock: &mut clocks::Clock, mut reg16: F, pc_state: &mut pc_state::PcState, original: u16) -> () {
    reg16(pc_state, original.wrapping_add(1));
    pc_state.increment_pc(1);
    clock.increment(6);
}

// INC (HL)
// Increment (HL) and set status flags.
pub fn inc_hl(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {

    let new_value =  memory.read(pc_state.get_hl()).wrapping_add(1);
    memory.write(pc_state.get_hl(), new_value);

    let mut f_value = pc_state.get_f();
    flagtables::calculate_inc_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(11);
}

////////////////////////////////////////////////////
// END Rust
////////////////////////////////////////////////////

// class OUT_n_A(Instruction):
//     def __init__(self, memory, pc_state, ports):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.ports    = ports
// 
// # OUT (n), self.pc_state.A
// # Write register A, to port n
//     def execute(self):
//      self.ports.portWrite(self.memory.read(self.pc_state.PC + 1), self.pc_state.A)
//      self.pc_state.PC+=2;
// 
//      return 11;
// 
// class RRCA(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.C = self.pc_state.A & 0x1;
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.A = (self.pc_state.A >> 1) | ((self.pc_state.A & 0x1) << 7);
//         self.pc_state.PC += 1
// 
//         return 4;
// 
// 
// class AND_r(Instruction):
//     def __init__(self, memory, pc_state, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.src = src
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A & self.src.get();
//         self.pc_state.PC += 1
//     
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAnd(self.pc_state.A);
//     
//         return 4;
// 
// class AND_a(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.PC += 1
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAnd(self.pc_state.A);
//         return 4;
// 
// class AND_n(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//     
//         self.pc_state.A = self.pc_state.A & self.memory.read(self.pc_state.PC +1);
//         self.pc_state.PC += 2;
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAnd(self.pc_state.A);
//     
//         return 7;
// 
// class OR_r(Instruction):
//     def __init__(self, memory, pc_state, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.src = src
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A | self.src.get();
//         self.pc_state.PC += 1
//     
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
//     
//         return 4;
// 
// class OR_a(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.PC += 1
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
//         return 4;
// 
// class OR_e(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A | self.pc_state.E
//         self.pc_state.PC += 1
//     
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
//     
//         return 4;
// 
// class XOR_r(Instruction):
//     def __init__(self, memory, pc_state, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.src = src
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A ^ self.src.get();
//         self.pc_state.PC += 1
//     
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
//     
//         return 4;
// 
// class XOR_a(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
//              ************* FLAGS *****************
//         self.status = flagtables.FlagTables.getStatusOr(0);
// 
//     def execute(self):
//         self.pc_state.A = 0
//         self.pc_state.PC += 1
//              ************* FLAGS *****************
//         self.pc_state.F.value = self.status
//     
//         return 4;
// 
// class EXX(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp16 = self.pc_state.BC;
//         self.pc_state.BC = self.pc_state.BC_;
//         self.pc_state.BC_ = tmp16;
//     
//         tmp16 = self.pc_state.DE;
//         self.pc_state.DE = self.pc_state.DE_;
//         self.pc_state.DE_ = tmp16;
//     
//         tmp16 = self.pc_state.HL;
//         self.pc_state.HL = self.pc_state.HL_;
//         self.pc_state.HL_ = tmp16;
//     
//         self.pc_state.PC += 1
//     
//         return 4;
// 
// class DJNZ(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     # DJNZ n
//     def execute(self):
//         cycles = 8;
//     
//         self.pc_state.B -= 1;
//         if (self.pc_state.B != 0):
// 
//             self.pc_state.PC += signed_char_to_int(self.memory.read(self.pc_state.PC + 1))
//             cycles += 5
//     
//         self.pc_state.PC += 2
//     
//         return cycles
//     
// 
// class RET(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.PCLow  = self.memory.read(self.pc_state.SP)
//         self.pc_state.SP += 1
//         self.pc_state.PCHigh = self.memory.read(self.pc_state.SP)
//         self.pc_state.SP += 1
//     
//         return 10;
// 
// # Addition instructions
// 
// class ADD16(Instruction):
//     def __init__(self, memory, pc_state, dst, add, cycles, pcInc = 1):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
//         self.add = add
//         self.cycles = cycles
//         self.pcInc = pcInc
// 
//     def execute(self):
//         a = self.dst.get()
//         b = self.add.get()
// 
//         r = (a & 0xFFF) + (b & 0xFFF);
//         if (r & 0x1000): # Half carry
//              ************* FLAGS *****************
//           self.pc_state.F.Fstatus.H = 1 # Half carry
//         else:
//           self.pc_state.F.Fstatus.H = 0 # Half carry
//         self.pc_state.F.Fstatus.N = 0;
//     
//         r = (a & 0xFFFF) + (b & 0xFFFF);
//         if (r & 0x10000): # Carry
//           self.pc_state.F.Fstatus.C = 1 # Carry
//         else:
//           self.pc_state.F.Fstatus.C = 0 # Carry
//     
//         self.dst.set(r)
//     
//         self.pc_state.PC += self.pcInc;
//     
//         return self.cycles;
// 
// class ADD_r(Instruction):
//     def __init__(self, memory, pc_state, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.src = src
// 
//     def execute(self):
//              ************* FLAGS *****************
//             self.pc_state.F.value = flagtables.FlagTables.getStatusAdd(self.pc_state.A,self.src.get());
//             self.pc_state.A = self.pc_state.A + self.src.get();
//             self.pc_state.PC += 1
//             return 4;
// 
// class SUB_r(Instruction):
//     def __init__(self, memory, pc_state, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.src = src
// 
//     def execute(self):
//              ************* FLAGS *****************
//             self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,self.src.get());
//             self.pc_state.A = self.pc_state.A - self.src.get();
//             self.pc_state.PC += 1
//             return 4;
// 
// class SUB_a(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//             self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,self.pc_state.A);
//             self.pc_state.A = 0
//             self.pc_state.PC += 1
//             return 4;
// 
// class BIT_r(Instruction):
//     def __init__(self, memory, pc_state, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.src = src
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.PC+1)
// 
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.Z = (self.src.get() >> ((tmp8 >> 3) & 7)) ^ 0x1;
//         self.pc_state.F.Fstatus.PV = flagtables.FlagTables.calculateParity(self.src.get());
//         self.pc_state.F.Fstatus.H = 1;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.F.Fstatus.S = 0;
//         self.pc_state.PC += 2;
//         return 8;
// 
// # self.pc_state.Bit b, (self.pc_state.HL) 
// class BIT_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.PC+1)
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.Z = (self.memory.read(self.pc_state.HL) >> 
//                             ((tmp8 >> 3) & 7)) ^ 0x1;
//         self.pc_state.F.Fstatus.H = 1;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.F.Fstatus.S = 0;
//         self.pc_state.PC += 2;
// 
//         return 12;
// 
// # RES b, r
// class RES_b_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.PC+1)
//         self.dst.set(int(self.dst) & ~(0x1 << ((tmp8 >> 3) & 7)));
//         self.pc_state.PC += 2;
//         return 8;
// 
// # RES b, HL
// class RES_b_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.PC+1)
//         self.memory.write(self.pc_state.HL, self.memory.read(self.pc_state.HL) & ~(0x1 << ((tmp8 >> 3) & 7)));
//         self.pc_state.PC += 2;
//         return 12;
// 
// # SET b, r
// class SET_b_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.PC+1)
//         self.dst.set(int(self.dst) | (0x1 << ((tmp8 >> 3) & 7)));
//         self.pc_state.PC += 2;
//         return 8;
// 
// # SET b, HL
// class SET_b_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.PC+1)
//         self.memory.write(self.pc_state.HL, self.memory.read(self.pc_state.HL) | (0x1 << ((tmp8 >> 3) & 7)));
//         self.pc_state.PC += 2;
//         return 12;
// 
// 
// class RLCA(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = (self.pc_state.A << 1) | ((self.pc_state.A >> 7) & 0x1);
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.C = self.pc_state.A & 0x1;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.PC += 1
//         return 4;
// 
// # RLC r
// class RLC_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         self.dst.set((int(self.dst) << 1) | ((int(self.dst) >> 7) & 0x1));
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(int(self.dst));
//         self.pc_state.F.Fstatus.C = int(self.dst) & 0x1; # bit-7 of src = bit-0
//         self.pc_state.PC+=2;
//         return 8;
// 
// # RLC (HL)
// class RLC_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.HL);
//         self.memory.write(self.pc_state.HL, (tmp8 << 1) | ((tmp8 >> 7) & 0x1));
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.memory.read(self.pc_state.HL));
//         self.pc_state.F.Fstatus.C = (tmp8 >> 7) & 0x1; # bit-7 of src
//         self.pc_state.PC+=2;
//         return 15;
// 
// # RRC r
// class RRC_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         self.dst.set((int(self.dst) >> 1) | ((int(self.dst) & 0x1) << 7));
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.dst);
//         self.pc_state.F.Fstatus.C = (int(self.dst) >> 7) & 0x1; # bit-0 of src
//         self.pc_state.PC+=2;
//         return 8
// 
// # RRC (HL)
// class RRC_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.HL);
//         self.memory.write(self.pc_state.HL,(tmp8 >> 1) | ((tmp8 & 0x1) << 7));
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.memory.read(self.pc_state.HL));
//         self.pc_state.F.Fstatus.C = tmp8 & 0x1; # bit-0 of src
//         self.pc_state.PC+=2;
//         return 8;
// 
// # RL r
// class RL_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = int(self.dst);
//              ************* FLAGS *****************
//         self.dst.set((int(self.dst) << 1) | (self.pc_state.F.Fstatus.C));
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(int(self.dst));
//         self.pc_state.F.Fstatus.C = (tmp8 >> 7) & 0x1;
//         self.pc_state.PC+=2;
//         return 8
// 
// # RR r
// class RR_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = int(self.dst);
//              ************* FLAGS *****************
//         self.dst.set((int(self.dst) >> 1) | (self.pc_state.F.Fstatus.C << 7));
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(int(self.dst));
//         self.pc_state.F.Fstatus.C = tmp8 & 0x1;
//         self.pc_state.PC+=2;
//         return 8;
// 
// # SLA r
// class SLA_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = (int(self.dst) >> 7) & 0x1;
//         self.dst.set(int(self.dst) << 1)
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(int(self.dst))
//         self.pc_state.F.Fstatus.C = tmp8;
// 
//         self.pc_state.PC += 2;
//         return 8
// 
// # SLA (HL)
// class SLA_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = (self.memory.read(self.pc_state.HL) >> 7) & 0x1;
//         self.memory.write(self.pc_state.HL, self.memory.read(self.pc_state.HL) << 1);
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.memory.read(self.pc_state.HL));
//         self.pc_state.F.Fstatus.C = tmp8;
// 
//         self.pc_state.PC += 2;
//         return 15
// 
// # SRA r
// class SRA_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = int(self.dst);
//         self.dst.set((int(self.dst) & 0x80) | ((int(self.dst) >> 1) & 0x7F));
// 
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.dst);
//         self.pc_state.F.Fstatus.C = tmp8 & 0x1;
// 
//         self.pc_state.PC += 2;
//         return 8
// 
// # SRA (HL)
// class SRA_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.HL);
//         self.memory.write(self.pc_state.HL, (tmp8 & 0x80) | ((tmp8 >> 1) & 0x7F));
// 
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.memory.read(self.pc_state.HL));
//         self.pc_state.F.Fstatus.C = tmp8 & 0x1;
// 
//         self.pc_state.PC += 2;
//         return 15;
// 
// # SLL r
// class SLL_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = (int(self.dst) >> 7) & 0x1;
//         self.dst.set(int(self.dst) << 1 | 0x1);
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(int(self.dst));
//         self.pc_state.F.Fstatus.C = tmp8;
// 
//         self.pc_state.PC += 2;
//         return 8
// 
// # SLL (HL)
// class SLL_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = (self.memory.read(self.pc_state.HL) >> 7) & 0x1;
//         self.memory.write(self.pc_state.HL, self.memory.read(self.pc_state.HL) << 1 | 0x1);
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.memory.read(self.pc_state.HL));
//         self.pc_state.F.Fstatus.C = tmp8;
// 
//         self.pc_state.PC += 2;
//         return 15
// 
// # SRL r
// class SRL_r(Instruction):
//     def __init__(self, memory, pc_state, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.dst = dst
// 
//     def execute(self):
//         tmp8 = int(self.dst);
//         self.dst.set((int(self.dst) >> 1) & 0x7F);
// 
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(int(self.dst));
//         self.pc_state.F.Fstatus.C = tmp8 & 0x1;
// 
//         self.pc_state.PC += 2;
//         return 8;
// 
// # SRL (HL)
// class SRL_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.HL);
//         self.memory.write(self.pc_state.HL, (tmp8 >> 1) & 0x7F);
// 
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.memory.read(self.pc_state.HL));
//         self.pc_state.F.Fstatus.C = tmp8 & 0x1;
// 
//         self.pc_state.PC += 2;
//         return 15;
// 
// class InstructionExec(object):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
// # LD (self.pc_state.IY+d), r
// class LD_IY_d_r(Instruction):
//     def __init__(self, memory, pc_state, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.src = src
// 
//     def execute(self):
//         self.memory.write(self.pc_state.IY + signed_char_to_int(self.memory.read(self.pc_state.PC+2)), self.src.get()); 
//         self.pc_state.PC += 3;
//         return 19
// 
// 
// # LD r, (nn)
// # Load the value from the 16-bit address into the 8-bit register
// class LD_r8_mem(Instruction):
//     # r - 8-bit
//     def __init__(self, memory, pc_state, r):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.r = r
// 
//     # Load the value at the address into a register.
//     def execute(self):
//         self.r.set(self.memory.read(self.memory.read16(self.pc_state.PC+1)));
//         self.pc_state.PC += 3;
// 
//         return 13;
// 
//     def get_cached_execute(self):
//         r = self.memory.read16(self.pc_state.PC+1)
//         s = self.r.set
//         pc = self.pc_state.PC + 3;
// 
//         def _get_cached_execute(self):
//             s(self.memory.read(r));
//             self.pc_state.PC = pc
//             return 13;
// 
//         return _get_cached_execute
// 
// # LD self.I_reg, nn
// class LD_I_nn(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.I_reg.set(self.memory.read16(self.pc_state.PC+2))
//         self.pc_state.PC += 4
//         return 20
//     
// # LD (nn), self.I_reg
// class LD_nn_I(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.memory.write(self.memory.read16(self.pc_state.PC+2), self.I_reg.get_low())
//         self.memory.write(self.memory.read16(self.pc_state.PC+2)+1, self.I_reg.get_high())
//         self.pc_state.PC += 4
//     
//         return 20
//     
// # LD self.I_reg, (nn)
// class LD_I__nn_(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.I_reg.set(self.memory.read16(self.memory.read16(self.pc_state.PC+2)))
//         self.pc_state.PC += 4
//     
//         return 20
//     
// # INC (self.I_reg+d)
// class INC_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp16 = self.I_reg.get() + signed_char_to_int(self.memory.read(self.pc_state.PC+2))
//         self.memory.write(tmp16, self.memory.read(tmp16) + 1)
//              ************* FLAGS *****************
//         self.pc_state.F.value = (self.pc_state.F.value & Instruction.FLAG_MASK_INC8) | flagtables.FlagTables.getStatusInc8(self.memory.read(tmp16))
//         self.pc_state.PC+=3
//         return 23
//     
// # self.pc_state.DEC (self.I_reg+d)
// class DEC_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp16 = self.I_reg.get() + signed_char_to_int(self.memory.read(self.pc_state.PC+2))
//         self.memory.write(tmp16, self.memory.read(tmp16) - 1)
//              ************* FLAGS *****************
//         self.pc_state.F.value = (self.pc_state.F.value & Instruction.FLAG_MASK_DEC8) | flagtables.FlagTables.getStatusDec8(self.memory.read(tmp16))
//         self.pc_state.PC+=3
//         return 23
//     
// # LD (self.I_reg + d), n
// class LD_I_d_n(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp16 = self.I_reg.get() + signed_char_to_int(self.memory.read(self.pc_state.PC+2))
//         self.memory.write(tmp16, self.memory.read(self.pc_state.PC+3))
//         self.pc_state.PC += 4
//         return  19
//     
// # LD r, (self.I_reg+e)
// class LD_r_I_e(Instruction):
//     def __init__(self, memory, pc_state, I_reg, dst):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
//         self.dst = dst
// 
//     def execute(self):
//         self.dst.set(self.memory.read(self.I_reg.get() + signed_char_to_int(self.memory.read(self.pc_state.PC+2))))
//                                         
//         self.pc_state.PC = self.pc_state.PC + 3
//         return  19
//     
// # LD (self.I_reg+d), r
// class LD_I_d_r(Instruction):
//     def __init__(self, memory, pc_state, I_reg, src):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
//         self.src = src
// 
//     def execute(self):
//                           
//         self.memory.write(self.I_reg.get() + signed_char_to_int(self.memory.read(self.pc_state.PC+2)), self.src.get()) 
//         self.pc_state.PC += 3
//         return  19
//     
// # self.pc_state.ADD self.pc_state.A,(self.I_reg+d)
// class ADDA_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.I_reg.get() + 
//                          signed_char_to_int(self.memory.read(self.pc_state.PC+2)))
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAdd(self.pc_state.A,tmp8)
//         self.pc_state.A = self.pc_state.A + tmp8
//         self.pc_state.PC += 3
//         return  19
//     
// # self.pc_state.ADC (self.I_reg + d)
// class ADC_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.A = add8c(self.pc_state, self.pc_state.A, self.memory.read(self.I_reg.get() + signed_char_to_int(self.memory.read(self.pc_state.PC+2))), self.pc_state.F.Fstatus.C)
//         self.pc_state.PC+=3
//         return 19
//     
// # SUB (self.I_reg + d)
// class SUB_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.I_reg.get() + 
//                          signed_char_to_int(self.memory.read(self.pc_state.PC+2)))
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,tmp8)
//         self.pc_state.A = self.pc_state.A - tmp8
//         self.pc_state.PC += 3
//         return  19
//     
// # self.pc_state.AND (self.I_reg + d)
// class AND_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A & self.memory.read(self.I_reg.get() +
//                          signed_char_to_int(self.memory.read(self.pc_state.PC+2)))
//         self.pc_state.PC+=3
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAnd(self.pc_state.A)
//     
//         return 19
//     
// # XOR (self.I_reg + d)
// class XOR_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A ^ self.memory.read(self.I_reg.get() +
//                          signed_char_to_int(self.memory.read(self.pc_state.PC+2)))
//         self.pc_state.PC+=3
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A)
//     
//         return  19
//     
// # OR (self.I_reg + d)
// class OR_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.I_reg.get() + 
//                          signed_char_to_int(self.memory.read(self.pc_state.PC+2)))
//         self.pc_state.A = self.pc_state.A | tmp8
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A)
//         self.pc_state.PC += 3
//         return  19
//     
// # CP (self.I_reg + d)
// class CP_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.I_reg.get() + 
//                          signed_char_to_int(self.memory.read(self.pc_state.PC+2)))
//         self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,tmp8)
//         self.pc_state.PC+=3
//         return 19
//     
// # Probably should turn this into a lookup table
// class BIT_I_d(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp16 = self.I_reg.get() + signed_char_to_int(self.memory.read(self.pc_state.PC+2))
//         tmp8  = self.memory.read(tmp16)
//         t8    = self.memory.read(self.pc_state.PC+3)
// 
//         if ((t8 & 0xC7) == 0x46): # self.pc_state.BIT b, (self.I_reg.get() + d)
//             tmp8 = (tmp8 >> ((t8 & 0x38) >> 3)) & 0x1
//              ************* FLAGS *****************
//             f = self.pc_state.F.Fstatus
//             f.Z = tmp8 ^ 0x1
//             f.PV = flagtables.FlagTables.calculateParity(tmp8)
//             f.H = 1
//             f.N = 0
//             f.S = 0
//         elif ((t8 & 0xC7) == 0x86): # RES b, (self.I_reg + d)
//             tmp8 = tmp8 & ~(0x1 << ((t8 >> 3) & 0x7))
//             self.memory.write(tmp16,tmp8)
//         elif ((t8 & 0xC7) == 0xC6): # SET b, (self.I_reg + d)
//             tmp8 = tmp8 | (0x1 << ((t8 >> 3) & 0x7))
//             self.memory.write(tmp16,tmp8)
//         else:
//             errors.error("Instruction arg for 0xFD 0xCB")
//     
//         self.pc_state.PC += 4
//     
//         return  23
// 
//     def get_cached_execute(self):
//         offset = signed_char_to_int(self.memory.read(self.pc_state.PC+2))
//         t8    = self.memory.read(self.pc_state.PC+3)
// 
//         if ((t8 & 0xC7) == 0x46): # self.pc_state.BIT b, (self.I_reg.get() + d)
//          t8_2 = (t8 & 0x38) >> 3
//          def _get_cached_execute(self):
//             tmp16 = self.I_reg.get() + offset
//             tmp8  = self.memory.read(tmp16)
// 
//             tmp8 = (tmp8 >> t8_2) & 0x1
// 
//             f = self.pc_state.F.Fstatus
//             f.Z = tmp8 ^ 0x1
//             f.PV = flagtables.FlagTables.calculateParity(tmp8)
//             f.H = 1
//             f.N = 0
//             f.S = 0
//     
//             self.pc_state.PC += 4
//     
//             return  23
//         elif ((t8 & 0xC7) == 0x86): # RES b, (self.I_reg + d)
//           t8_2 = ~(0x1 << ((t8 >> 3) & 0x7))
//           def _get_cached_execute(self):
//             tmp16 = self.I_reg.get() + offset
//             tmp8  = self.memory.read(tmp16) & t8_2
//             self.memory.write(tmp16,tmp8)
//     
//             self.pc_state.PC += 4
//     
//             return  23
// 
//         elif ((t8 & 0xC7) == 0xC6): # SET b, (self.I_reg + d)
//           t8_2 =(0x1 << ((t8 >> 3) & 0x7))
//           def _get_cached_execute(self):
//             tmp16 = self.I_reg.get() + offset
//             tmp8  = self.memory.read(tmp16) | t8_2
//             self.memory.write(tmp16,tmp8)
//     
//             self.pc_state.PC += 4
//     
//             return  23
// 
//         return _get_cached_execute
//     
// # POP self.I_reg
// class POP_I(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.I_reg.set_low(self.memory.read(self.pc_state.SP))
//         self.pc_state.SP += 1
//         self.I_reg.set_high(self.memory.read(self.pc_state.SP))
//         self.pc_state.SP += 1
//         self.pc_state.PC += 2
//         return  14
//     
// # EX (self.pc_state.SP), self.I_reg
// class EX_SP_I(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.SP)
//         self.memory.write(self.pc_state.SP, self.I_reg.get_low())
//         self.I_reg.set_low(tmp8)
//         tmp8 = self.memory.read(self.pc_state.SP+1)
//         self.memory.write(self.pc_state.SP+1, self.I_reg.get_high())
//         self.I_reg.set_high(tmp8)
//         self.pc_state.PC+=2
//         return  23
//     
// # PUSH self.I_reg
// class PUSH_I(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.I_reg.get_high())
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.I_reg.get_low())
//         self.pc_state.PC += 2
//     
//         return 15
//     
// # Don't know how many self.clocks.cycles
// # LD self.pc_state.PC, self.I_reg
// class LD_PC_I(Instruction):
//     def __init__(self, memory, pc_state, I_reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.I_reg = I_reg
// 
//     def execute(self):
//         self.pc_state.PC = self.I_reg.get()
//         return 6
// 
// # IN r, (C)
// class IN_r_C(Instruction):
//     def __init__(self, memory, pc_state, ports, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
//         self.ports = ports
// 
//     def execute(self):
//         self.reg.set(self.ports.portRead(self.pc_state.C))
//         self.pc_state.PC += 2;
//         return 12;
//     
// # OUT (C), r
// class OUT_C_r(Instruction):
//     def __init__(self, memory, pc_state, ports, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
//         self.ports = ports
// 
//     def execute(self):
//         self.ports.portWrite(self.pc_state.C, self.reg.get());
//         self.pc_state.PC += 2;
//         return 3;
//     
// # SBC_HL_r16
// class SBC_HL_r16(Instruction):
//     def __init__(self, memory, pc_state, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.HL = sub16c(self.pc_state, self.pc_state.HL, int(self.reg), self.pc_state.F.Fstatus.C);
//     
//         self.pc_state.PC += 2;
//         return  15;
//     
// # LD (nn), self.pc_state.BC
// class LD_nn_BC(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.memory.write(self.memory.read16(self.pc_state.PC+2), self.pc_state.C);
//         self.memory.write(self.memory.read16(self.pc_state.PC+2)+1, self.pc_state.B);
//         self.pc_state.PC += 4;
//     
//         return  20;
//     
// # NEG
// class NEG(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusSub(0,self.pc_state.A);
//         self.pc_state.A = -self.pc_state.A;
//         self.pc_state.PC += 2;
//         return 8;
//     
// # LD I, self.pc_state.A
// class LD_I_A(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.I = self.pc_state.A;
//         self.pc_state.PC += 2;
//         return  9;
//     
// # Load 16-bit self.pc_state.BC register
// # LD self.pc_state.BC, (nn)
// class LD_BC_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.BC = self.memory.read16(self.memory.read16(self.pc_state.PC+2)); 
//         self.pc_state.PC += 4;
//         return  20;
//     
// # Fself.pc_state.IXME, should check, since there is only one
// # interupting device, this is the same as normal ret
// # RETI
// class RETI(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//         self.pc_state.SP += 1
//         self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//         self.pc_state.SP += 1
//     
//         return  14;
//                 
// # LD (nn), self.pc_state.DE
// class LD_nn_DE(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.memory.write(self.memory.read16(self.pc_state.PC+2), self.pc_state.E);
//         self.memory.write(self.memory.read16(self.pc_state.PC+2)+1, self.pc_state.D);
//         self.pc_state.PC += 4;
//     
//         return  20;
//     
// # LD self.pc_state.A, I
// class LD_A_I(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.I;
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.F.Fstatus.PV = self.pc_state.IFF2;
//         self.pc_state.F.Fstatus.S = (self.pc_state.A & 0x80) >> 7;
//         if (self.pc_state.A == 0):
//             self.pc_state.F.Fstatus.Z = 1
//         else:
//             self.pc_state.F.Fstatus.Z = 0
//     
//         self.pc_state.PC += 2;
//         return  9;
//     
// # LD self.pc_state.DE, (nn)    
// class LD_DE_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.DE = self.memory.read16(self.memory.read16(self.pc_state.PC+2));
//         self.pc_state.PC += 4;
//         return  20;
//     
// # Fself.pc_state.IXME, not sure about this
// # LD self.pc_state.A, R
// class LD_A_R(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         # HMM??? Random???
//         self.pc_state.R =  (self.pc_state.R & 0x80) | ((self.pc_state.R + 1) & 0x7F);
//         self.pc_state.A = self.pc_state.R;
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.F.Fstatus.PV = self.pc_state.IFF2;
//         self.pc_state.F.Fstatus.S = (self.pc_state.A & 0x80) >> 7;
//         if (self.pc_state.A == 0):
//             self.pc_state.F.Fstatus.Z = 1
//         else:
//             self.pc_state.F.Fstatus.Z = 0
//     
//         self.pc_state.PC += 2;
//         return  9;
//     
// # Fself.pc_state.IXME, can't find existance of this instruction
// # RRD, wacky instruction
// class RRD(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.pc_state.A;
//         self.pc_state.A = (self.pc_state.A & 0xF0) | (self.memory.read(self.pc_state.HL) & 0xF);
//         self.memory.write(self.pc_state.HL, 
//                ((self.memory.read(self.pc_state.HL) >> 4) & 0xF) | 
//                ((tmp8 << 4) & 0xF0));
//     
//              ************* FLAGS *****************
//         tmp8 = self.pc_state.F.Fstatus.C;
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
//         self.pc_state.F.Fstatus.C = tmp8;
//     
//         self.pc_state.PC+=2;
//         return  18;
//     
// # self.pc_state.ADC self.pc_state.HL, self.pc_state.r16
// class ADC_HL_r16(Instruction):
//     def __init__(self, memory, pc_state, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.HL = add16c(self.pc_state, self.pc_state.HL, int(self.reg), self.pc_state.F.Fstatus.C);
//         self.pc_state.PC+=2;
//         return 15;
//     
// # Fself.pc_state.IXME, not sure about the existance of this instruction
// # LD self.pc_state.HL, (nn)
// class LD_HL_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.HL = self.memory.read16(self.memory.read16(self.pc_state.PC+2));
//         self.pc_state.PC += 4;
//     
//         return  20;
//     
// # LD (nn), self.pc_state.SP
// class LD_nn_SP(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.memory.write(self.memory.read16(self.pc_state.PC+2), self.pc_state.SPLow);
//         self.memory.write(self.memory.read16(self.pc_state.PC+2)+1, self.pc_state.SPHigh);
//         self.pc_state.PC += 4;
//     
//         return  6;
//     
// # Load 16-bit self.pc_state.BC register
// # LD self.pc_state.SP, (nn)
// class LD_SP_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.SP = self.memory.read16(self.memory.read16(self.pc_state.PC+2)); 
//         self.pc_state.PC += 4;
//         return  20;
//     
// # LDI
// class LDI(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.memory.write(self.pc_state.DE, self.memory.read(self.pc_state.HL));
//         self.pc_state.DE += 1
//         self.pc_state.HL += 1
//         self.pc_state.BC -= 1
//         if (self.pc_state.BC == 0):
//              ************* FLAGS *****************
//             self.pc_state.F.Fstatus.PV = 1
//         else:
//             self.pc_state.F.Fstatus.PV = 0
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.PC += 2;
//     
//         return  16;
//     
// # CPI
// class CPI(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,self.memory.read(self.pc_state.HL));
//         self.pc_state.HL += 1
//         self.pc_state.BC -= 1
//         if (self.pc_state.BC == 0):
//             self.pc_state.F.Fstatus.PV = 1
//         else:
//             self.pc_state.F.Fstatus.PV = 0
//         self.pc_state.PC += 2;
//         return  16;
//     
// # INI
// class INI(Instruction):
//     def __init__(self, memory, pc_state, ports):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.ports = ports
// 
//     def execute(self):
//         self.pc_state.B -= 1
//         self.memory.write(self.pc_state.HL, self.ports.portRead(self.pc_state.C));
//         self.pc_state.HL += 1
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.N = 1;
//         if (self.pc_state.B == 0):
//             self.pc_state.F.Fstatus.Z = 1;
//         else:
//             self.pc_state.F.Fstatus.Z = 0;
//     
//         self.pc_state.PC += 2;
//         return  16;
//     
// # OUTI
// class OUTI(Instruction):
//     def __init__(self, memory, pc_state, ports):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.ports = ports
// 
//     def execute(self):
//         self.pc_state.B -= 1
//         self.ports.portWrite(self.pc_state.C, self.memory.read(self.pc_state.HL));
//         self.pc_state.HL += 1
//         if (self.pc_state.B == 0):
//              ************* FLAGS *****************
//             self.pc_state.F.Fstatus.Z = 1
//         else:
//             self.pc_state.F.Fstatus.Z = 0
//         self.pc_state.F.Fstatus.N = 1;
//         self.pc_state.PC += 2;
//         return  16;
//     
// # OUTD
// class OUTD(Instruction):
//     def __init__(self, memory, pc_state, ports):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.ports = ports
// 
//     def execute(self):
//         self.pc_state.B -= 1
//         self.ports.portWrite(self.pc_state.C, self.memory.read(self.pc_state.HL));
//         self.pc_state.HL -= 1
//         if (self.pc_state.B == 0):
//              ************* FLAGS *****************
//             self.pc_state.F.Fstatus.Z = 1
//         else:
//             self.pc_state.F.Fstatus.Z = 0
//         self.pc_state.F.Fstatus.N = 1;
//         self.pc_state.PC += 2;
//         return  16;
//     
// # LDIR
// class LDIR(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         if (self.pc_state.BC >= 4):
//             self.memory.writeMulti(self.pc_state.DE, self.pc_state.HL, 4);
//             self.pc_state.DE += 4;
//             self.pc_state.HL += 4;
//             self.pc_state.BC -= 4;
//             cycles += 84;
//         else:
//             self.pc_state.BC -= 1
//             self.memory.write(self.pc_state.DE, self.memory.read(self.pc_state.HL));
//             self.pc_state.DE += 1
//             self.pc_state.HL += 1
//             cycles += 21;
//     
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.F.Fstatus.PV = 0;
//         self.pc_state.F.Fstatus.N = 1; # hmmm, not sure
//         if (self.pc_state.BC == 0):
//             self.pc_state.F.Fstatus.N = 0;
//             self.pc_state.PC += 2;
//             cycles -=5;
// 
//         return cycles
//     
// # CPIR
// class CPIR(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.BC -= 1
//              ************* FLAGS *****************
//         tmp8 = self.pc_state.F.Fstatus.C;
//         self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,self.memory.read(self.pc_state.HL));
//         self.pc_state.HL += 1
//         self.pc_state.F.Fstatus.C = tmp8; 
//     
//         if ((self.pc_state.BC == 0)or(self.pc_state.F.Fstatus.Z == 1)):
//             self.pc_state.F.Fstatus.PV = 0; 
//             self.pc_state.PC += 2;
//             cycles += 16;
//         else:
//             self.pc_state.F.Fstatus.PV = 1; 
//             cycles += 21;
// 
//         return cycles
//     
// # Should speed this function up a bit
// # Flags match emulator, not z80 document
// # OTIR (port)
// class OTIR(Instruction):
//     def __init__(self, memory, pc_state, ports):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.ports = ports
// 
//     def execute(self):
//         cycles = 0
//         if (self.pc_state.B >= 8):
//             self.pc_state.B -= 8;
//             self.ports.portMultiWrite(self.pc_state.C, self.memory.readArray(self.pc_state.HL,8), 8);
//             self.pc_state.HL+= 8;
//             cycles += 168;
//         else:
//             self.pc_state.B -= 1
//             self.ports.portWrite(self.pc_state.C, self.memory.read(self.pc_state.HL));
//             self.pc_state.HL += 1
//             cycles += 21;
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.S = 0; # Unknown
//         self.pc_state.F.Fstatus.H = 0; # Unknown
//         self.pc_state.F.Fstatus.PV = 0; # Unknown
//         self.pc_state.F.Fstatus.N = 1;
//         self.pc_state.F.Fstatus.Z = 0;
//         if (self.pc_state.B == 0):
//             self.pc_state.F.Fstatus.Z = 1;
//             self.pc_state.PC += 2;
//             cycles -= 5;
//         return cycles
//     
// # LDDR
// class LDDR(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.memory.write(self.pc_state.DE, self.memory.read(self.pc_state.HL));
//         self.pc_state.DE -= 1
//         self.pc_state.HL -= 1
//         self.pc_state.BC -= 1
//         if (self.pc_state.BC == 0):
//             self.pc_state.PC += 2;
//             cycles += 16;
//              ************* FLAGS *****************
//             self.pc_state.F.Fstatus.N = 0;
//             self.pc_state.F.Fstatus.H = 0;
//             self.pc_state.F.Fstatus.PV = 0;
//         else:
//             cycles += 21;
// 
//         return cycles
// 
// 
// ################ NEW INSTRUCTIONS ##################
// 
// # EX self.pc_state.AF, self.pc_state.AF'
// class EX(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmpa = self.pc_state.A;
//              ************* FLAGS *****************
//         tmpf = self.pc_state.F.value;
//         self.pc_state.A = self.pc_state.A_;
//         self.pc_state.F.value = self.pc_state.F_;
//         self.pc_state.A_ = tmpa;
//         self.pc_state.F_ = tmpf;
// 
//         self.pc_state.PC += 1
//         return 4;
// 
// # RLA
// class RLA(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.pc_state.A;
//              ************* FLAGS *****************
//         self.pc_state.A = (self.pc_state.A << 1) | (self.pc_state.F.Fstatus.C);
//         self.pc_state.F.Fstatus.C = (tmp8 & 0x80) >> 7;
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.PC += 1
//         return 4;
// 
// # RRA
// class RRA(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.pc_state.A;
//         self.pc_state.A = (self.pc_state.A >> 1) | (self.pc_state.F.Fstatus.C << 7);
//         self.pc_state.F.Fstatus.C = tmp8 & 0x1;
//         self.pc_state.F.Fstatus.H = 0;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.PC += 1
//         return 4;
// 
// # LD (nn), self.pc_state.HL
// class LD__nn_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.memory.write(self.memory.read16(self.pc_state.PC+1), self.pc_state.L);
//         self.memory.write(self.memory.read16(self.pc_state.PC+1)+1, self.pc_state.H);
//         self.pc_state.PC += 3;
// 
//         return  16;
// 
// # Really need to put this into a table
// class DAA(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.N == 0): # self.pc_state.Addition instruction
//             calculateDAAAdd(self.pc_state);
//         else: # Subtraction instruction
//             calculateDAASub(self.pc_state);
//         self.pc_state.PC += 1
//         return 4;
// 
// # CPL
// class CPL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.H = 1;
//         self.pc_state.F.Fstatus.N = 1;
//         self.pc_state.A ^= 0xFF;
//         self.pc_state.PC += 1
// 
//         return 4;
// 
// # SCF
// class SCF(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//          self.pc_state.F.Fstatus.H = 0;
//          self.pc_state.F.Fstatus.N = 0;
//          self.pc_state.F.Fstatus.C = 1;
//          self.pc_state.PC += 1
//          return  4;
// 
// # CCF
// class CCF(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.Fstatus.H = self.pc_state.F.Fstatus.C;
//         self.pc_state.F.Fstatus.N = 0;
//         self.pc_state.F.Fstatus.C = 1-self.pc_state.F.Fstatus.C; #Invert carry flag
//         self.pc_state.PC += 1
//         return  4;
// 
// # self.pc_state.ADD (self.pc_state.HL) 
// class ADD_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAdd(self.pc_state.A,self.memory.read(self.pc_state.HL));
//         self.pc_state.A = self.pc_state.A + self.memory.read(self.pc_state.HL);
//         self.pc_state.PC += 1
//         return 7;
// 
// # self.pc_state.ADC r
// class ADC_r(Instruction):
//     def __init__(self, memory, pc_state, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.A = add8c(self.pc_state, self.pc_state.A, self.reg.get(), self.pc_state.F.Fstatus.C);
//         self.pc_state.PC += 1
//         return 4;
// 
// # self.pc_state.ADC (self.pc_state.HL)
// class ADC_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.A = add8c(self.pc_state, self.pc_state.A, self.memory.read(self.pc_state.HL), self.pc_state.F.Fstatus.C);
//         self.pc_state.PC += 1
//         return 7;
// 
// # SUB (self.pc_state.HL) 
// class SUB_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,self.memory.read(self.pc_state.HL));
//         self.pc_state.A = self.pc_state.A - self.memory.read(self.pc_state.HL);
//         self.pc_state.PC += 1
//         return 7;
// 
// # SBC r
// class SBC_A_r(Instruction):
//     def __init__(self, memory, pc_state, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.A = sub8c(self.pc_state, self.pc_state.A, self.reg.get(), self.pc_state.F.Fstatus.C);
//         self.pc_state.PC += 1
//         return 4;
// 
// # SBC (self.pc_state.HL)
// class SBC_A_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = sub8c(self.pc_state, self.pc_state.A, self.memory.read(self.pc_state.HL), self.pc_state.F.Fstatus.C);
//         self.pc_state.PC += 1
//         return 7;
// 
// # self.pc_state.AND (self.pc_state.HL)
// class AND_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A & self.memory.read(self.pc_state.HL);
//         self.pc_state.PC += 1
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAnd(self.pc_state.A);
// 
//         return 7;
// 
// # XOR (self.pc_state.HL)
// class XOR_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A ^ self.memory.read(self.pc_state.HL);
//         self.pc_state.PC += 1
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
// 
//         return  7;
// 
// # OR (self.pc_state.HL)
// class OR_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A | self.memory.read(self.pc_state.HL);
//         self.pc_state.PC += 1
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
// 
//         return  7;
// 
// # RET NZ
// class RET_NZ(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.Z == 0):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles +=5;
//         return cycles
// 
// # CALL NZ, nn
// class CALL_NZ_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.Z == 0):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
//             cycles += 7;
// 
//         cycles += 10;
//         return cycles
// 
// # PUSH 
// class PUSH(Instruction):
//     def __init__(self, memory, pc_state, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
// 
//     def execute(self):
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.reg.get_high());
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.reg.get_low());
//         self.pc_state.PC += 1
// 
//         return 11;
// 
// # self.pc_state.ADD n
// class ADD_n(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusAdd(self.pc_state.A,self.memory.read(self.pc_state.PC + 1));
//         self.pc_state.A = self.pc_state.A + self.memory.read(self.pc_state.PC + 1);
//         self.pc_state.PC+=2;
//         return 7;
// 
// # RST
// class RST(Instruction):
//     def __init__(self, memory, pc_state, rst_addr):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.rst_addr = rst_addr
// 
//     def execute(self):
//         self.pc_state.PC += 1
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
// 
//         self.pc_state.PC = self.rst_addr
// 
//         return  11;
// 
// # RET Z
// class RST_Z(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.Z == 1):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles +=5;
//         return cycles
// 
// # CALL Z, nn
// class CALL_Z_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.Z == 1):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
// 
//             cycles += 7;
//         else:
//             cycles += 10;
//         return cycles
// 
// # CALL nn
// class CALL_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.PC += 3;
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//         self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
// 
//         return  17;
// 
// # self.pc_state.ADC nn
// class ADC_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.A = add8c(self.pc_state, self.pc_state.A, self.memory.read(self.pc_state.PC + 1), self.pc_state.F.Fstatus.C);
//         self.pc_state.PC+=2;
//         return 4;
// 
// # RET NC
// class RET_NC(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.C == 0):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles +=5;
//         return cycles
// 
// # POP self.pc_state.DE
// class POP(Instruction):
//     def __init__(self, memory, pc_state, reg):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.reg = reg
// 
//     def execute(self):
//         self.reg.set_low(self.memory.read(self.pc_state.SP))
//         self.pc_state.SP += 1
//         self.reg.set_high(self.memory.read(self.pc_state.SP));
//         self.pc_state.SP += 1
//         self.pc_state.PC += 1
//         return  10;
// 
// # CALL NC, nn  
// class CALL_NC_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.C == 0):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
// 
//             cycles += 7;
//         else:
//             cycles += 10;
//         return cycles
// 
// # SUB n
// class SUB_n(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusSub(self.pc_state.A,self.memory.read(self.pc_state.PC + 1));
//         self.pc_state.A = self.pc_state.A - self.memory.read(self.pc_state.PC + 1);
//         self.pc_state.PC += 2;
//         return  7;
// 
// # RET C
// class RET_C(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.C == 1):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles+=5;
//         return cycles
// 
// # Call C, nn
// class CALL_C_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.C == 1):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
//             cycles += 17;
//         else:
//             cycles += 10;
//         return cycles
// 
// # SBC n 
// class SBC_n(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//              ************* FLAGS *****************
//         self.pc_state.A = sub8c(self.pc_state, self.pc_state.A, self.memory.read(self.pc_state.PC + 1), self.pc_state.F.Fstatus.C);
//         self.pc_state.PC+=2;
//         return 7;
// 
// # RET PO  
// class RET_PO(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.PV == 0):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles +=5;
//         return cycles
// 
// 
// # EX (self.pc_state.SP), self.pc_state.HL
// class EX_SP_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp8 = self.memory.read(self.pc_state.SP);
//         self.memory.write(self.pc_state.SP, self.pc_state.L);
//         self.pc_state.L = tmp8;
//         tmp8 = self.memory.read(self.pc_state.SP+1);
//         self.memory.write(self.pc_state.SP+1, self.pc_state.H);
//         self.pc_state.H = tmp8;
//         self.pc_state.PC += 1
//         return  19;
// 
// # CALL PO, nn 
// class CALL_PO_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.PV == 0):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
//             cycles += 7;
// 
//         cycles += 10;
//         return cycles
// 
// # RET PE  
// class RET_PE(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.PV == 1):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles +=5;
//         return cycles
// 
// # EX self.pc_state.DE, self.pc_state.HL
// class EX_DE_HL(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         tmp16 = self.pc_state.DE;
//         self.pc_state.DE = self.pc_state.HL;
//         self.pc_state.HL = tmp16;
//         self.pc_state.PC += 1
//         return 4;
// 
// # CALL PE, nn
// class CALL_PE_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.PV == 1):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
//             cycles += 7;
// 
//         cycles += 10;
//         return cycles
// 
// # XOR n
// class XOR_n(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A ^ self.memory.read(self.pc_state.PC + 1);
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
//         self.pc_state.PC+=2;
//         return 7;
// 
// # RET P, if Positive
// class RET_P(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.S == 0):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles +=5;
//         return cycles
// 
// # POP self.pc_state.AF
// class POP_AF(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.F.value = self.memory.read(self.pc_state.SP);
//         self.pc_state.SP += 1
//         self.pc_state.A = self.memory.read(self.pc_state.SP);
//         self.pc_state.SP += 1
// 
//         self.pc_state.PC += 1
// 
//         return 10;
// 
// # Disable interupts
// # DI
// class DI(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.IFF1 = 0;
//         self.pc_state.IFF2 = 0;
//         self.pc_state.PC += 1
// 
//         return 4;
// 
// # CALL P, nn  if Positive
// class CALL_P_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.S == 0):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
//             cycles += 7;
// 
//         cycles += 10;
//         return cycles
// 
// # PUSH self.pc_state.AF
// class PUSH_AF(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.SP -= 1
//         self.memory.write(self.pc_state.SP, self.pc_state.A);
//         self.pc_state.SP -= 1
//              ************* FLAGS *****************
//         self.memory.write(self.pc_state.SP, self.pc_state.F.value);
//         self.pc_state.PC += 1
// 
//         return 11;
// 
//     def get_cached_execute(self):
//         ps = self.pc_state
//         w = self.memory.write
//         def _get_cached_execute(self):
//             ps.SP -= 1
//             w(ps.SP, ps.A);
//             ps.SP -= 1
//              ************* FLAGS *****************
//             w(ps.SP, ps.F.value);
//             ps.PC += 1
// 
//             return 11;
//         return _get_cached_execute
// 
// # OR n
// class OR_n(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         self.pc_state.A = self.pc_state.A | self.memory.read(self.pc_state.PC + 1);
//              ************* FLAGS *****************
//         self.pc_state.F.value = flagtables.FlagTables.getStatusOr(self.pc_state.A);
//         self.pc_state.PC += 2;
//         return 7;
// 
// # RET M  if Negative
// class RET_M(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.S == 1):
//             self.pc_state.PCLow  = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             self.pc_state.PCHigh = self.memory.read(self.pc_state.SP);
//             self.pc_state.SP += 1
//             cycles += 11;
//         else:
//             self.pc_state.PC += 1
//             cycles +=5;
//         return cycles
// 
// # Enable interupts
// # EI
// class EI(Instruction):
//     def __init__(self, memory, clocks, pc_state, interupt, poll_interupts, step):
//         self.memory = memory
//         self.pc_state = pc_state
//         self.interupt = interupt
//         self.poll_interupts = poll_interupts
//         self.step = step
//         self.clocks = clocks
// 
//     def execute(self):
//         self.pc_state.PC += 1
// 
//         # Process next instruction before enabling interupts
//         self.step(); # Single step
// 
//         self.pc_state.IFF1 = 1;
//         self.pc_state.IFF2 = 1;
// 
//           # Check for any pending interupts
//         if (self.poll_interupts(self.clocks.cycles) == True):
//             self.interupt()
// 
//         return 4
// 
// # CALL M, nn  if Negative
// class CALL_M_nn(Instruction):
//     def __init__(self, memory, pc_state):
//         self.memory = memory
//         self.pc_state = pc_state
// 
//     def execute(self):
//         cycles = 0
//         self.pc_state.PC += 3;
//              ************* FLAGS *****************
//         if (self.pc_state.F.Fstatus.S == 1):
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCHigh);
//             self.pc_state.SP -= 1
//             self.memory.write(self.pc_state.SP, self.pc_state.PCLow);
//             self.pc_state.PC = self.memory.read16(self.pc_state.PC-2);
// 
//             cycles += 7;
//         else:
//             cycles += 10;
//         return cycles

#[cfg(test)]
mod tests {
    use crate::sega::cpu::instruction_set;
    use crate::sega::cpu::pc_state;

    #[test]
    fn test_add_sub_functions() {
        let mut pc_state = pc_state::PcState::new();
        assert_eq!(instruction_set::signed_char_to_int(-1 as i8), -1 as i16);

        assert_eq!(instruction_set::add8c(&mut pc_state, 0, 0, false), 0);
        assert_eq!(pc_state.get_f().get_z(), 1);

        assert_eq!(instruction_set::add8c(&mut pc_state, 0, 0, true), 1);
        assert_eq!(pc_state.get_f().get_z(), 0);

        assert_eq!(instruction_set::add8c(&mut pc_state, 0x7, 0x9, true), 0x11);
        assert_eq!(pc_state.get_f().get_z(), 0);
        assert_eq!(pc_state.get_f().get_h(), 1);
        assert_eq!(pc_state.get_f().get_n(), 0);

        assert_eq!(instruction_set::add8c(&mut pc_state, 0xFF, 0xFF, true), 0xFF);
        assert_eq!(pc_state.get_f().get_z(), 0);
        assert_eq!(pc_state.get_f().get_c(), 1);
        assert_eq!(pc_state.get_f().get_pv(), 0);

        assert_eq!(instruction_set::sub8c(&mut pc_state, 0xFF, 0xFF, true), 0xFF);
        assert_eq!(instruction_set::sub8c(&mut pc_state, 0x7F, 0xFF, true), 0x7F);
        assert_eq!(pc_state.get_f().get_pv(), 0);
        assert_eq!(pc_state.get_f().get_c(), 0);
        assert_eq!(pc_state.get_f().get_n(), 1);

        assert_eq!(instruction_set::sub8c(&mut pc_state, 0xFF, 0x2, true), 0xFC);
        assert_eq!(pc_state.get_f().get_pv(), 0);
        assert_eq!(pc_state.get_f().get_c(), 1);

        assert_eq!(instruction_set::add16c(&mut pc_state, 0xFFFF, 0xFFFF, true), 0xFFFF);
        assert_eq!(instruction_set::add16c(&mut pc_state, 0, 0, false), 0);
        assert_eq!(pc_state.get_f().get_z(), 1);
        assert_eq!(pc_state.get_f().get_n(), 0);

        assert_eq!(instruction_set::add16c(&mut pc_state, 0x3FFF, 0x7001, true), 0xB001);
        assert_eq!(pc_state.get_f().get_h(), 1);

        assert_eq!(instruction_set::sub16c(&mut pc_state, 0x0000, 0x000F, true), 0xFFF0);
        assert_eq!(pc_state.get_f().get_n(), 1);
    }
}
