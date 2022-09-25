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
pub fn in_a_n<M>(clock: &mut clocks::Clock, memory: &mut M, 
              pc_state: &mut pc_state::PcState, ports: &mut ports::Ports) -> () where M: memory::MemoryRW {

    pc_state.set_a(ports.port_read(memory.read(pc_state.get_pc() + 1)));
    pc_state.increment_pc(2);
    clock.increment(11);
}

//0xD3
// OUT (N), A
pub fn out_n_a<M>(clock: &mut clocks::Clock, memory: &mut M, 
              pc_state: &mut pc_state::PcState, ports: &mut ports::Ports) -> () where M: memory::MemoryRW {

    ports.port_write(memory.read(pc_state.pc_reg.get() + 1), pc_state.get_a());
    pc_state.increment_pc(2);
    clock.increment(11);
}


//0xF3, disable interrupts
pub fn ei(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    // The 'step' and program counter increment is currently implemented
    // 'outside' of this function.

    pc_state.set_iff1(true);
    pc_state.set_iff2(true);
    clock.increment(4);
}

//0xF3, disable interrupts
pub fn di(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    pc_state.set_iff1(false);
    pc_state.set_iff2(false);
    pc_state.increment_pc(1);
    clock.increment(4);
}

pub fn signed_char_to_int(v: i8) -> i16 {
    return v as i16;
}

pub fn shift_right_arithmetic(input:u8) -> (u8, bool) 
{
    // Shifts to the right, maintaining the sign bit.
    let bit0 = input & 0b1 == 1;

    let result = (input >> 1) | (input & 0x80 as u8);
    (result, bit0)
}

pub fn shift_right_logical(input:u8) -> (u8, bool) 
{
    // Shifts to the right
    let bit0 = input & 0b1 == 1;

    let result = input >> 1;
    (result, bit0)
}

pub fn shift_left_arithmetic(input:u8) -> (u8, bool) 
{
    // Rotates to the right, bit 7 goes into carry & result
    let bit7 = (input >> 7) & 0b1 == 1;

    let result = input << 1;
    (result, bit7)
}

pub fn shift_left_logical(input:u8) -> (u8, bool) 
{
    // Rotates to the right, bit 7 goes into carry & result
    let bit7 = (input >> 7) & 0b1 == 1;

    let result = (input << 1) | 0x1;
    (result, bit7)
}

pub fn rotate_right_carry(input:u8) -> (u8, bool) 
{
    // Rotates to the right, bit 0 goes into carry & result
    let bit0 = input & 0b1 == 1;

    let result = (input >> 1) | ((bit0 as u8) << 7);
    (result, bit0)
}

pub fn rotate_right(input:u8, carry: bool) -> (u8, bool) 
{
    // Rotates to the right, bit 0 goes into carry, carry into bit 7
    let bit0 = input & 0b1 == 1;

    let result = (input >> 1) | ((carry as u8) << 7);
    (result, bit0)
}

pub fn rotate_left_carry(input:u8) -> (u8, bool) 
{
    // Rotates to the right, bit 7 goes into carry & result
    let bit7 = (input >> 7) & 0b1 == 1;

    let result = (input << 1) | (bit7 as u8);
    (result, bit7)
}

pub fn rotate_left(input:u8, carry: bool) -> (u8, bool) 
{
    // Rotates to the left, bit 7 goes into carry, carry into bit 0
    let bit7 = (input >> 7) & 0b1 == 1;

    let result = (input << 1) | (carry as u8);
    (result, bit7)
}

pub fn add8<F16>(a:u8, b:u8, af_reg: &mut F16) -> u8 
    where F16: pc_state::FlagReg {

    // Just call the add c function.
    add8c(a, b, false, af_reg)
}

pub fn sub8<F16>(a:u8, b:u8, af_reg: &mut F16) -> u8 
    where F16: pc_state::FlagReg {

    // Just call the sub c function.
    sub8c(a, b, false, af_reg)
}

pub fn add8c<F16>(a:u8, b:u8, c:bool, af_reg: &mut F16) -> u8 
    where F16: pc_state::FlagReg {

    let mut f_status = af_reg.get_flags();
    let result = status_flags::u8_carry(a, b, c, &mut f_status);
    f_status.set_n(0); // Clear N to indicate add
    af_reg.set_flags(&f_status);

    result
}

pub fn cp_flags<F16>(a:u8, b:u8, af_reg: &mut F16) -> ()
    where F16: pc_state::FlagReg {

    // CP flags calculated set the same as for subtaction, but the result is ignored.
    sub8c(a, b, false, af_reg);
}

// Subtract two 8 bit ints and the carry bit, set flags accordingly
fn sub8c<F16>(a:u8, b:u8, c:bool, af_reg: &mut F16) -> u8
    where F16: pc_state::FlagReg {

    let mut f_status = af_reg.get_flags();
    // a - b + c -> a + (~b + 1) + c -> a + ~b - c
    let result = status_flags::u8_carry(a, !b, !c, &mut f_status);
    f_status.set_n(1); // Set N to indicate subtract
    af_reg.set_flags(&f_status);

    result
}

pub fn add16c<F16>(a:u16, b:u16, c:bool, af_reg: &mut F16) -> u16
    where F16: pc_state::FlagReg {

    let mut f_status = af_reg.get_flags();
    let result = status_flags::u16_carry(a, b, c, &mut f_status);
    f_status.set_n(0);
    af_reg.set_flags(&f_status);

    result
}

pub fn sub16c<F16>(a:u16, b:u16, c:bool, af_reg: &mut F16) -> u16
    where F16: pc_state::FlagReg {

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
pub fn ld_16_nn<M, R16> (clock: &mut clocks::Clock, memory: &mut M, 
              pc_reg: &mut R16, r16_reg: &mut R16) -> () where M: memory::MemoryRW,
                                                             R16: pc_state::Reg16RW {
    r16_reg.set(memory.read16(pc_reg.get() +1)); 

    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(10);
}

// LD (16 REG), r
// eg: LD (HL), r
// Load the 8-bit register, r, 16-bit address
pub fn ld_mem_r<M, R16>(clock: &mut clocks::Clock, memory: &mut M, 
                r: u8, pc_reg: &mut R16, address_reg: &R16) -> () where M: memory::MemoryRW, 
                                                                      R16: pc_state::Reg16RW {
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
pub fn ld_r_mem<M, F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, mut dst_fn: F, addr_reg_value: u16) -> () where M: memory::MemoryRW {
    dst_fn(pc_state, memory.read(addr_reg_value));
    pc_state.increment_pc(1);
    clock.increment(7);
}

// LD r,n
pub fn ld_r<M, F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, mut dst_fn: F) -> () where M: memory::MemoryRW {
    dst_fn(pc_state, memory.read(pc_state.get_pc() + 1));
    pc_state.increment_pc(2);
    clock.increment(7);
}

// LD r, (nn)
// Load the value from the 16-bit address into the 16-bit register
pub fn ld_r16_mem<M, R16>(clock: &mut clocks::Clock, memory: &mut M, 
              pc_reg: &mut R16, r16_reg: &mut R16) -> () where M: memory::MemoryRW, 
                                                             R16: pc_state::Reg16RW {
    r16_reg.set(memory.read16(memory.read16(pc_reg.get()+1)));
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(20);
}

// LD (16 REG), n
// eg LD (HL), n
// Load the value 'n' into the 16-bit address
pub fn ld_mem_n<M, R16>(clock: &mut clocks::Clock, memory: &mut M, 
              pc_reg: &mut R16, r16_reg: &mut R16) -> () where M: memory::MemoryRW, 
                                                             R16: pc_state::Reg16RW {
    // Load the 8 bit value 'n' into memory.
    memory.write(r16_reg.get(), memory.read(pc_reg.get() + 1));
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(10);
}

// LD r, (nn)
// eg LD A, (nn)  (is actually the only version)
// Op Code: 3A
// Load the value from the 16-bit address into the 8-bit register
pub fn ld_r8_mem<M, F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, mut dst_fn: F) -> () where M: memory::MemoryRW {
    dst_fn(pc_state, memory.read(memory.read16(pc_state.get_pc() +1 )));
    pc_state.increment_pc(3);
    clock.increment(13);
}

//  
//  LD SP, HL Load a 16-bit register with the value from another 16-bit register
pub fn ld_sp_hl<R16>(clock: &mut clocks::Clock, hl_reg: &R16, 
                pc_reg: &mut R16, sp_reg: &mut R16) -> () where R16: pc_state::Reg16RW {
    sp_reg.set(hl_reg.get()); 

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(6);
}

// LD (nn), r
// eg LD (nn), A   - Which is the only version of this function.
pub fn ld_nn_r<M, R16>(clock: &mut clocks::Clock, memory: &mut M, r: u8,
              pc_reg: &mut R16) -> () where M: memory::MemoryRW, 
                                          R16: pc_state::Reg16RW {
    memory.write(memory.read16(pc_reg.get()+1), r);
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(13);
}


/*************************************************************************************/
/* Compare Instructions                                                              */
/*************************************************************************************/

// CP n
// Compare accumulator with 'n' to set status flags (but don't change accumulator)
pub fn cp_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state.get_a(),  memory.read(pc_state.get_pc() +1), &mut pc_state.af_reg);

    pc_state.increment_pc(2);
    clock.increment(7);
}

// CP r
// Compare accumulator with register r to set status flags (but don't change accumulator)
pub fn cp_r(clock: &mut clocks::Clock, r: u8, pc_state: &mut pc_state::PcState) -> () {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state.get_a(),  r, &mut pc_state.af_reg);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// CP (hl)
// Compare accumulator with the value from (HL) to set status flags (but don't change accumulator)
pub fn cp_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    // This function sets the 'pc_state.f'
    cp_flags(pc_state.get_a(), memory.read(pc_state.get_hl()), &mut pc_state.af_reg);

    pc_state.increment_pc(1);
    clock.increment(7);
}

/*************************************************************************************/
/* JUMP Instructions                                                                 */
/*************************************************************************************/

pub fn jp_nn<M>(clock: &mut clocks::Clock, memory: &mut M, 
              pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    pc_state.set_pc(memory.read16(pc_state.get_pc() + 1));
    clock.increment(10);
}

//  JP (HL)
// Load PC with HL, to jump to that location.
pub fn jp_hl<R16>(clock: &mut clocks::Clock, hl_reg: &R16, pc_reg: &mut R16) -> () where R16: pc_state::Reg16RW {
    pc_reg.set(hl_reg.get()); 
    clock.increment(4);
}

// Jump relative condition.  (instruction grouping isn't as convinient as for 'JP cc, nn')
// JR cc, e 
pub fn jr_cc_e<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, condition: bool) -> () where M: memory::MemoryRW {
    clock.increment(7);

    if condition {
        pc_state.increment_pc(memory.read(pc_state.get_pc() + 1) as i8);
        clock.increment(5);
    }
    pc_state.increment_pc(2);
}

// JR NZ, e
pub fn jrnz_e<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_z() == 0);
}

// JR Z, e
pub fn jrz_e<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_z() == 1);
}

// JR NC, e
pub fn jrnc_e<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_c() == 0);
}

// JR C, e
pub fn jrc_e<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    jr_cc_e(clock, memory, pc_state, pc_state.get_f().get_c() == 1);
}

// Relative jump
// JR e
pub fn jr_e<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    // Timing for this is the same as for conditional jump relative.
    jr_cc_e(clock, memory, pc_state, true);
}

// Absolute Jump on condition
// JP cc, nn
pub fn jump_cc_nn<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, condition:bool) -> () where M: memory::MemoryRW {
    if condition {
        pc_state.set_pc(memory.read16(pc_state.get_pc() + 1));
        clock.increment(5);
    } else {
        pc_state.increment_pc(3);
    }

    clock.increment(10);
}

pub fn djnz<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {

    pc_state.set_b(pc_state.get_b().wrapping_sub(1));
    if pc_state.get_b() != 0 {
        pc_state.increment_pc(memory.read(pc_state.pc_reg.get() + 1) as i8);
        clock.increment(13);
    } else{
        clock.increment(8);
    }

    pc_state.increment_pc(2);
}

// Note, could also add '#[derive(Copy, Clone)]' to 'Reg16'
fn exchange_reg<R16>(first: &mut R16, second: &mut R16) -> () where R16: pc_state::Reg16RW {
    let tmp_high = first.get_high();
    let tmp_low  = first.get_low();
    first.set_high(second.get_high());
    first.set_low(second.get_low());
    second.set_low(tmp_low);
    second.set_high(tmp_high);
}

// EXX
// Exchange shadow registers.
pub fn exx(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {

    exchange_reg(&mut pc_state.bc_reg, &mut pc_state.shadow_bc_reg);
    exchange_reg(&mut pc_state.de_reg, &mut pc_state.shadow_de_reg);
    exchange_reg(&mut pc_state.hl_reg, &mut pc_state.shadow_hl_reg);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// EX
// Exchange AF, AF', or DE, HL registers
pub fn ex<R1, R2>(clock: &mut clocks::Clock, pc_reg: &mut R1, first: &mut R2, second: &mut R2) -> () 
    where R1: pc_state::Reg16RW,
          R2: pc_state::Reg16RW,
{
    exchange_reg(first, second);
    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(4);
}

// EX (SP), HL
pub fn ex_sp_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () 
    where M: memory::MemoryRW
{

    let mut tmp8 = memory.read(pc_state.sp_reg.get());
    memory.write(pc_state.sp_reg.get(), pc_state.get_l());
    pc_state.set_l(tmp8);

    tmp8 = memory.read(pc_state.sp_reg.get() + 1);
    memory.write(pc_state.sp_reg.get() + 1, pc_state.get_h());
    pc_state.set_h(tmp8);

    pc_state.increment_pc(1);
    clock.increment(19);
}

// Call on condition
// CALL cc, nn
pub fn call_cc_nn<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, condition:bool) -> () where M: memory::MemoryRW {
    pc_state.increment_pc(3);
    if condition {
        pc_state.increment_sp(-1);
        memory.write(pc_state.sp_reg.get(), pc_state.pc_reg.high);
        pc_state.increment_sp(-1);
        memory.write(pc_state.sp_reg.get(), pc_state.pc_reg.low);
        pc_state.pc_reg.set(memory.read16(pc_state.pc_reg.get() - 2));
        clock.increment(17);
    } else {
        clock.increment(10);
    }
}

// Call on condition
// CALL nn
pub fn call_nn<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    // Call is the same as a conditional call that's always true.
    call_cc_nn(clock, memory, pc_state, true);
}

// RST (0x0, 0x8, 0x10, 0x18, 0x20, 0x30, 0x38)
pub fn rst<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, rst_addr: u8) -> () where M: memory::MemoryRW {

    pc_state.increment_pc(1);
    pc_state.increment_sp(-1);
    memory.write(pc_state.sp_reg.get(), pc_state.get_pc_high());
    pc_state.increment_sp(-1);
    memory.write(pc_state.sp_reg.get(), pc_state.get_pc_low());
    
    pc_state.set_pc(rst_addr as u16);
    
    clock.increment(11);
}

/**********************************************************/
/* INC/DEC                                                */
/**********************************************************/

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
pub fn dec_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {

    let new_value =  memory.read(pc_state.get_hl()).wrapping_sub(1);
    memory.write(pc_state.get_hl(), new_value);

    let mut f_value = pc_state.get_f();
    status_flags::calculate_dec_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(11);
}

// DEC (IX+d), INC (IY+d), 
pub fn dec_i_d<M, F16, R16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, 
                  af_reg: &mut F16, i16_reg: &R16) -> () where M: memory::MemoryRW,     
                                                                             R16: pc_state::Reg16RW,
                                                                             F16: pc_state::FlagReg {

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
pub fn inc_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {

    let new_value =  memory.read(pc_state.get_hl()).wrapping_add(1);
    memory.write(pc_state.get_hl(), new_value);

    let mut f_value = pc_state.get_f();
    status_flags::calculate_inc_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(11);
}

// LD (nn), HL
pub fn ld_mem_nn_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    memory.write(memory.read16(pc_state.get_pc()+1), pc_state.get_l());
    memory.write(memory.read16(pc_state.get_pc()+1)+1, pc_state.get_h());

    pc_state.increment_pc(3);
    clock.increment(16);
}

// LD (nn), HL (Extended)
// same as ld_mem_nn_hl, but part of the extended group?
pub fn ld_mem_nn_hl_extended<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
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

pub fn add_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let result = add8(pc_state.get_a(), memory.read(pc_state.hl_reg.get()), &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(7);
}

pub fn adc_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let carry = pc_state.get_f().get_c();
    let result = add8c(pc_state.get_a(), memory.read(pc_state.hl_reg.get()), carry==1, &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(7);
}

pub fn add_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let result = add8(pc_state.get_a(), memory.read(pc_state.pc_reg.get()+1), &mut pc_state.af_reg);
    let mut f_status = pc_state.get_f();
    status_flags::and_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(2);
    clock.increment(7);
}

pub fn adc_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let carry = pc_state.get_f().get_c();
    let result = add8c(pc_state.get_a(), memory.read(pc_state.pc_reg.get()+1), carry == 1, &mut pc_state.af_reg);
    let mut f_status = pc_state.get_f();
    status_flags::and_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(2);
    clock.increment(7);
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

pub fn sub_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let result = sub8(pc_state.get_a(), memory.read(pc_state.hl_reg.get()), &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(7);
}

pub fn sbc_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let carry = pc_state.get_f().get_c();
    let result = sub8c(pc_state.get_a(), memory.read(pc_state.hl_reg.get()), carry==1, &mut pc_state.af_reg);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(7);
}

pub fn sub_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let result = sub8(pc_state.get_a(), memory.read(pc_state.pc_reg.get()+1), &mut pc_state.af_reg);
    let mut f_status = pc_state.get_f();
    status_flags::and_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(2);
    clock.increment(7);
}

pub fn sbc_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let carry = pc_state.get_f().get_c();
    let result = sub8c(pc_state.get_a(), memory.read(pc_state.pc_reg.get()+1), carry == 1, &mut pc_state.af_reg);
    let mut f_status = pc_state.get_f();
    status_flags::and_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(2);
    clock.increment(7);
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

pub fn and_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let result = pc_state.get_a() & memory.read(pc_state.hl_reg.get());
    let mut f_status = pc_state.get_f();
    status_flags::and_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(7);
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

pub fn xor_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let result = pc_state.get_a() ^ memory.read(pc_state.hl_reg.get());
    let mut f_status = pc_state.get_f();
    status_flags::xor_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(7);
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

pub fn or_hl<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW  {

    let result = pc_state.get_a() | memory.read(pc_state.hl_reg.get());
    let mut f_status = pc_state.get_f();
    status_flags::or_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(1);
    clock.increment(7);
}


pub fn and_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW{
    let result = pc_state.get_a() & memory.read(pc_state.pc_reg.get() +1);
    let mut f_status = pc_state.get_f();
    status_flags::and_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(2);
    clock.increment(7);
}

pub fn xor_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW{

    let result = pc_state.get_a() ^ memory.read(pc_state.pc_reg.get() +1);
    let mut f_status = pc_state.get_f();
    status_flags::xor_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(2);
    clock.increment(7);
}

pub fn or_n<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW{

    let result = pc_state.get_a() | memory.read(pc_state.pc_reg.get() +1);
    let mut f_status = pc_state.get_f();
    status_flags::or_flags(&mut f_status, result); 
    pc_state.set_f(f_status);
    pc_state.set_a(result);

    pc_state.increment_pc(2);
    clock.increment(7);
}

////////////////////////////////////////////////////
// 16-bit arithmetic Group
////////////////////////////////////////////////////

// ADD HL, ss
pub fn add16<R16, F16>(clock: &mut clocks::Clock, src_value: u16, 
             pc_reg: &mut R16, hl_reg: &mut R16, af_reg: &mut F16) -> () 
    where R16: pc_state::Reg16RW, 
          F16: pc_state::FlagReg {

    hl_reg.set(add16c(hl_reg.get(), src_value, false, af_reg));

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(11);
}

////////////////////////////////////////////////////
// Rotate and shift group
////////////////////////////////////////////////////

// RRCA
// Rotate Right with carry
pub fn rrca<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let (new_value, carry) =  rotate_right_carry(src);

    dst_fn(pc_state, new_value);
    let mut f_value = pc_state.get_f();
    status_flags::set_rotate_accumulator_flags(carry, &mut f_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// RRA
// Rotate Right 
pub fn rra<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let mut f_value = pc_state.get_f();
    let (new_value, carry) =  rotate_right(src, f_value.get_c()==1);

    dst_fn(pc_state, new_value);
    status_flags::set_rotate_accumulator_flags(carry, &mut f_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// RLCA
// Rotate Left with carry
pub fn rlca<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let (new_value, carry) =  rotate_left_carry(src);

    dst_fn(pc_state, new_value);
    let mut f_value = pc_state.get_f();
    status_flags::set_rotate_accumulator_flags(carry, &mut f_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// RLA
// Rotate Left 
pub fn rla<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let mut f_value = pc_state.get_f();
    let (new_value, carry) =  rotate_left(src, f_value.get_c()==1);

    dst_fn(pc_state, new_value);
    status_flags::set_rotate_accumulator_flags(carry, &mut f_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// RET
pub fn ret<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
    pc_state.set_pc_low(memory.read(pc_state.sp_reg.get()));
    pc_state.increment_sp(1);
    pc_state.set_pc_high(memory.read(pc_state.sp_reg.get()));
    pc_state.increment_sp(1);

    clock.increment(10);
}

// RET cc,  Return conditionally.
pub fn ret_cc<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, condition: bool) -> () where M: memory::MemoryRW {

    if condition {
        pc_state.set_pc_low(memory.read(pc_state.sp_reg.get()));
        pc_state.increment_sp(1);
        pc_state.set_pc_high(memory.read(pc_state.sp_reg.get()));
        pc_state.increment_sp(1);
        clock.increment(11);
    }
    else
    {
        pc_state.increment_pc(1);
        clock.increment(5);
    }
}

// POP qq 
// Pop 2 bytes off the stack into a 16-bit register.
pub fn pop<M, R1, R2>(clock: &mut clocks::Clock, memory: &mut M, pc_reg : &mut R1, sp_reg : &mut R1, dst_reg : &mut R2) -> ()
    where M: memory::MemoryRW,
          R1: pc_state::Reg16RW,
          R2: pc_state::Reg16RW,
{
    dst_reg.set_low(memory.read(sp_reg.get()));
    pc_state::PcState::increment_reg(sp_reg, 1);
    dst_reg.set_high(memory.read(sp_reg.get()));
    pc_state::PcState::increment_reg(sp_reg, 1);
    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(10);
}

// PUSH qq 
// Pop 2 bytes onto the stack into a 16-bit register.
pub fn push<M, R1, R2>(clock: &mut clocks::Clock, memory: &mut M, pc_reg : &mut R1, sp_reg : &mut R1, src_reg : &R2) -> ()
    where M: memory::MemoryRW,
          R1: pc_state::Reg16RW,
          R2: pc_state::Reg16RW, // Flag register is implemented as a different type, so need more than 1 type.
{
    pc_state::PcState::increment_reg(sp_reg, -1);
   memory.write(sp_reg.get(), src_reg.get_high());
    pc_state::PcState::increment_reg(sp_reg, -1);
    memory.write(sp_reg.get(), src_reg.get_low());

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(11);
}

pub fn daa(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> ()
{
    if pc_state.get_f().get_n() == 0 { // perform addition
        calculate_daa_add(pc_state);
    } else { // perform Subtraction
        calculate_daa_sub(pc_state);
    }

    pc_state.increment_pc(1);
    clock.increment(4);
}
/*************************************************************************************/
/* General purpose arithmetic and CPU control                                        */
/*************************************************************************************/

// CPL
// one's complement of 'A'
pub fn cpl(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> ()
{
    let mut f_status = pc_state.get_f();
    f_status.set_h(1);
    f_status.set_n(1);
    pc_state.set_f(f_status);
    pc_state.set_a(pc_state.get_a() ^0xFF);

    pc_state.increment_pc(1);
    clock.increment(4);
}

// SCF
// Set the carry flag
pub fn scf<R16, F16>(clock: &mut clocks::Clock, pc_reg : &mut R16, af_reg: &mut F16) -> () 
    where R16: pc_state::Reg16RW,
          F16: pc_state::FlagReg,
{
    let mut f_status = af_reg.get_flags();
    f_status.set_h(0);
    f_status.set_n(0);
    f_status.set_c(1);
    af_reg.set_flags(&f_status);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(4);
}

// CCF
// Compliment/Invert the carry flag
pub fn ccf<R16, F16>(clock: &mut clocks::Clock, pc_reg : &mut R16, af_reg: &mut F16) -> () 
    where R16: pc_state::Reg16RW,
          F16: pc_state::FlagReg,
{
    let mut f_status = af_reg.get_flags();
    f_status.set_h(f_status.get_c());
    f_status.set_n(0);
    f_status.set_c(f_status.get_c() ^ 1);
    af_reg.set_flags(&f_status);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(4);
}

pub fn halt(clock: &mut clocks::Clock) -> () {
    // Should 'loop/no-op' until an inerrupt occurs (or just immediately trigger one).
    clock.increment(4);
    panic!("halt not correctly implemented");
}

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
