use super::pc_state;
use super::super::memory::memory;
use super::super::ports;
use super::super::clocks;
use super::status_flags;

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
    pc_state.set_iff1(false);
    pc_state.set_iff2(false);
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

fn add8(a:u8, b:u8, af_reg: &mut pc_state::FlagReg16) -> u8 {
    // Just call the add c function.
    add8c(a, b, false, af_reg)
}

fn sub8(a:u8, b:u8, af_reg: &mut pc_state::FlagReg16) -> u8 {
    // Just call the sub c function.
    sub8c(a, b, false, af_reg)
}

fn add8c(a:u8, b:u8, c:bool, af_reg: &mut pc_state::FlagReg16) -> u8 {
    let mut f_status = af_reg.get_flags();
    let result = status_flags::u8_carry(a, b, c, &mut f_status);
    f_status.set_n(0); // Clear N to indicate add
    af_reg.set_flags(&f_status);

    result
}

pub fn cp_flags(a:u8, b:u8, af_reg: &mut pc_state::FlagReg16) -> () {
    // CP flags calculated set the same as for subtaction, but the result is ignored.
    sub8c(a, b, false, af_reg);
}

// Subtract two 8 bit ints and the carry bit, set flags accordingly
fn sub8c(a:u8, b:u8, c:bool, af_reg: &mut pc_state::FlagReg16) -> u8 {
    let mut f_status = af_reg.get_flags();
    // a - b + c -> a + (~b + 1) + c -> a + ~b - c
    let result = status_flags::u8_carry(a, !b, !c, &mut f_status);
    f_status.set_n(1); // Set N to indicate subtract
    af_reg.set_flags(&f_status);

    result
}

pub fn add16c(a:u16, b:u16, c:bool, af_reg: &mut pc_state::FlagReg16) -> u16 {
    let mut f_status = af_reg.get_flags();
    let result = status_flags::u16_carry(a, b, c, &mut f_status);
    f_status.set_n(0);
    af_reg.set_flags(&f_status);

    result
}

fn sub16c(a:u16, b:u16, c:bool, af_reg: &mut pc_state::FlagReg16) -> u16 {
    let mut f_status = af_reg.get_flags();
    // a - b + c -> a + (~b + 1) + c -> a + ~b - c
    let result = status_flags::u16_carry(a, !b, !c, &mut f_status);
    f_status.set_n(1);
    af_reg.set_flags(&f_status);

    result
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

    f_status.set_pv(u8::from(status_flags::calculate_parity(pc_state.get_a())));
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

    f_status.set_pv(u8::from(status_flags::calculate_parity(pc_state.get_a())));
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
              pc_reg: &mut dyn pc_state::Reg16RW, r16_reg: &mut dyn pc_state::Reg16RW) -> () {
    r16_reg.set(memory.read16(pc_reg.get() +1)); 

    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(10);
}

// LD (16 REG), r
// eg: LD (HL), r
// Load the 8-bit register, r, 16-bit address
pub fn ld_mem_r(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
                r: u8, pc_reg: &mut dyn pc_state::Reg16RW, address_reg: &dyn pc_state::Reg16RW) -> () {
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
              pc_reg: &mut dyn pc_state::Reg16RW, r16_reg: &mut dyn pc_state::Reg16RW) -> () {
    r16_reg.set(memory.read16(memory.read16(pc_reg.get()+1)));
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(20);
}

// LD (16 REG), n
// eg LD (HL), n
// Load the value 'n' into the 16-bit address
pub fn ld_mem_n(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, 
              pc_reg: &mut dyn pc_state::Reg16RW, r16_reg: &mut dyn pc_state::Reg16RW) -> () {
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
pub fn ld_sp_hl(clock: &mut clocks::Clock, hl_reg: &dyn pc_state::Reg16RW, 
                pc_reg: &mut dyn pc_state::Reg16RW, sp_reg: &mut dyn pc_state::Reg16RW) -> () {
    sp_reg.set(hl_reg.get()); 

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(6);
}

// LD (nn), r
// eg LD (nn), A   - Which is the only version of this function.
pub fn ld_nn_r(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, r: u8,
              pc_reg: &mut dyn pc_state::Reg16RW) -> () {
    memory.write(memory.read16(pc_reg.get()+1), r);
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(13);
}


/*************************************************************************************/
/* Compare Instructions                                                              */
/*************************************************************************************/

// CP n
// Compare accumulator with 'n' to set status flags (but don't change accumulator)
pub fn cp_n(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state.get_a(),  memory.read(pc_state.get_pc() +1), &mut pc_state.af_reg);

    pc_state.increment_pc(2);
    clock.increment(7);
}

// CP r
// Compare accumulator with register r to set status flags (but don't change accumulator)
pub fn cp_r(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, r: u8, pc_state: &mut pc_state::PcState) -> () {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state.get_a(),  r, &mut pc_state.af_reg);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// CP (hl)
// Compare accumulator with the value from (HL) to set status flags (but don't change accumulator)
pub fn cp_hl(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state.get_a(), memory.read(pc_state.get_hl()), &mut pc_state.af_reg);

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
pub fn jp_hl(clock: &mut clocks::Clock, hl_reg: &dyn pc_state::Reg16RW, pc_reg: &mut dyn pc_state::Reg16RW) -> () {
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
    status_flags::calculate_dec_flags(&mut f_value, new_value);
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
    status_flags::calculate_dec_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(11);
}

// DEC (IX+d), INC (IY+d), 
pub fn dec_i_d(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_reg: &mut dyn pc_state::Reg16RW, af_reg: &mut pc_state::FlagReg16, i16_reg: &dyn pc_state::Reg16RW) -> () {

    let address = i16_reg.get().wrapping_add(memory.read(pc_reg.get() + 2) as u16);
    let new_value =  memory.read(address).wrapping_sub(1);

    memory.write(address, new_value);

    let mut f_value = af_reg.get_flags();
    status_flags::calculate_dec_flags(&mut f_value, new_value);
    af_reg.set_flags(&f_value);

    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(23);
}

// INC r
// Increment register and set status flags.
pub fn inc_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let new_value =  src.wrapping_add(1);
    dst_fn(pc_state, new_value);

    let mut f_value = pc_state.get_f();
    status_flags::calculate_inc_flags(&mut f_value, new_value);
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
    status_flags::calculate_inc_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(11);
}

// INC (IX+d), INC (IY+d), 
pub fn inc_i_d(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_reg: &mut dyn pc_state::Reg16RW, af_reg: &mut pc_state::FlagReg16, i16_reg: &dyn pc_state::Reg16RW) -> () {

    let address = i16_reg.get().wrapping_add(memory.read(pc_reg.get() + 2) as u16);
    let new_value =  memory.read(address).wrapping_add(1);

    memory.write(address, new_value);

    let mut f_value = af_reg.get_flags();
    status_flags::calculate_inc_flags(&mut f_value, new_value);
    af_reg.set_flags(&f_value);

    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(23);
}

// LD (nn), HL
pub fn ld_mem_nn_hl(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    memory.write(memory.read16(pc_state.get_pc()+1), pc_state.get_l());
    memory.write(memory.read16(pc_state.get_pc()+1)+1, pc_state.get_h());

    pc_state.increment_pc(3);
    clock.increment(16);
}

// LD (nn), HL (Extended)
// same as ld_mem_nn_hl, but part of the extended group?
pub fn ld_mem_nn_hl_extended(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_state: &mut pc_state::PcState) -> () {
    memory.write(memory.read16(pc_state.get_pc()+2), pc_state.get_l());
    memory.write(memory.read16(pc_state.get_pc()+2)+1, pc_state.get_h());

    pc_state.increment_pc(4);
    clock.increment(20);
}

////////////////////////////////////////////////////
// 8-bit arithmetic Group
////////////////////////////////////////////////////

pub fn add_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {

    let result = add8(pc_state.get_a(), r, &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(4);
}

pub fn adc_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {

    let carry = pc_state.get_f().get_c();
    let result = add8c(pc_state.get_a(), r, carry==1, &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(4);
}


pub fn sub_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {

    let result = sub8(pc_state.get_a(), r, &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(4);
}

pub fn sbc_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {

    let carry = pc_state.get_f().get_c();
    let result = sub8c(pc_state.get_a(), r, carry==1, &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(4);
}


pub fn and_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {

    let result = pc_state.get_a() & r;
    let mut f_status = pc_state.get_f();
    status_flags::and_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(4);
}

pub fn xor_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {

    let result = pc_state.get_a() ^ r;
    let mut f_status = pc_state.get_f();
    status_flags::xor_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(4);
}

pub fn or_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {

    let result = pc_state.get_a() | r;
    let mut f_status = pc_state.get_f();
    status_flags::or_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(4);
}


////////////////////////////////////////////////////
// 16-bit arithmetic Group
////////////////////////////////////////////////////

// ADD HL, ss
pub fn add16(clock: &mut clocks::Clock, src_value: u16, 
             pc_reg: &mut dyn pc_state::Reg16RW, hl_reg: &mut dyn pc_state::Reg16RW, af_reg: &mut pc_state::FlagReg16) -> () {

    hl_reg.set(add16c(hl_reg.get(), src_value, false, af_reg));

    pc_state::PcState::increment_reg(pc_reg, 1);
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

        assert_eq!(instruction_set::add8c(0, 0, false, &mut pc_state.af_reg), 0);
        assert_eq!(pc_state.get_f().get_z(), 1);

        assert_eq!(instruction_set::add8c(0, 0, true, &mut pc_state.af_reg), 1);
        assert_eq!(pc_state.get_f().get_z(), 0);

        assert_eq!(instruction_set::add8c(0x7, 0x9, true, &mut pc_state.af_reg), 0x11);
        assert_eq!(pc_state.get_f().get_z(), 0);
        assert_eq!(pc_state.get_f().get_h(), 1);
        assert_eq!(pc_state.get_f().get_n(), 0);

        assert_eq!(instruction_set::add8c(0xFF, 0xFF, true, &mut pc_state.af_reg), 0xFF);
        assert_eq!(pc_state.get_f().get_z(), 0);
        assert_eq!(pc_state.get_f().get_c(), 1);
        assert_eq!(pc_state.get_f().get_pv(), 0);

        assert_eq!(instruction_set::sub8c(0xFF, 0xFF, true, &mut pc_state.af_reg), 0xFF);
        assert_eq!(instruction_set::sub8c(0x7F, 0xFF, true, &mut pc_state.af_reg), 0x7F);
        assert_eq!(pc_state.get_f().get_pv(), 0);
        assert_eq!(pc_state.get_f().get_c(), 0);
        assert_eq!(pc_state.get_f().get_n(), 1);

        assert_eq!(instruction_set::sub8c(0xFF, 0x2, true, &mut pc_state.af_reg), 0xFC);
        assert_eq!(pc_state.get_f().get_pv(), 0);
        assert_eq!(pc_state.get_f().get_c(), 1);

        assert_eq!(instruction_set::add16c( 0xFFFF, 0xFFFF, true, &mut pc_state.af_reg), 0xFFFF);
        assert_eq!(instruction_set::add16c(0, 0, false, &mut pc_state.af_reg), 0);
        assert_eq!(pc_state.get_f().get_z(), 1);
        assert_eq!(pc_state.get_f().get_n(), 0);

        assert_eq!(instruction_set::add16c(0x3FFF, 0x7001, true, &mut pc_state.af_reg), 0xB001);
        assert_eq!(pc_state.get_f().get_h(), 1);

        assert_eq!(instruction_set::sub16c(0x0000, 0x000F, true, &mut pc_state.af_reg), 0xFFF0);
        assert_eq!(pc_state.get_f().get_n(), 1);
    }
}
