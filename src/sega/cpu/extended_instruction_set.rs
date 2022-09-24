// This module is intended to hold all of the 'extended' instructions.
// Basically all instructions that require 2 op-codes to decode
// 
// 0xCB
// 0xDD
// 0xFD
// 0xED
//
// The main reason to separate, is to make it easier to adjust PC offset/timing settings later.
// Initially, setting each one to do it's own offset/increments.

use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::status_flags;
use super::instruction_set;

/*************************************************************************************/
/* Extended Load Instructions                                                        */
/*************************************************************************************/

// LD (IX+d), r; LD (IY+d), 
// op code:  0xDD, 0b01110rrr, 0bdddddddd
// op code:  0xFD, 0b01110rrr, 0bdddddddd
// pub fn ld_iy_d_r
// pub fn ld_ix_d_r
pub fn ld_i_d_r<M, R16>(clock: &mut clocks::Clock, memory: &mut M, r: u8, pc_reg: &mut R16, i16_reg: &R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW {

    memory.write(i16_reg.get().wrapping_add((memory.read(pc_reg.get()+2) as i8) as u16), r); 
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(19);
}

// LD I, nn
// LD IX, nn; LD IY, nn
pub fn ld_i_nn<M, R16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, i16_reg: &mut R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW {

    i16_reg.set(memory.read16(pc_reg.get() + 2));
    pc_state::PcState::increment_reg(pc_reg, 4);
    clock.increment(20);
}

// LD I, (nn)
// LD IX, (nn); LD IY, (nn)
// was ld_i__nn
pub fn ld_i_mem_nn<M, R16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, i16_reg: &mut R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW {

    i16_reg.set(memory.read16(memory.read16(pc_reg.get()+2)));
    pc_state::PcState::increment_reg(pc_reg, 4);
    clock.increment(20);
}

// LD (nn), HL (Extended)
// same as ld_nn_hl, but part of the extended group?
// pub fn ld_nn_hl_extended
// pub fn ld_nn_hl
// pub fn ld_nn_I
pub fn ld_mem_nn_reg16<M, R16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, reg16: &pc_state::Reg16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW {

    memory.write(memory.read16(pc_reg.get()+2),   reg16.low);
    memory.write(memory.read16(pc_reg.get()+2)+1, reg16.high);

    pc_state::PcState::increment_reg(pc_reg, 4);
    clock.increment(20);
}

// LD dd, (nn)
// 0b00dd0001 -> BC 00, DE 01, HL 10, SP 11
// 0bnnnnnnnn
// 0bnnnnnnnn
pub fn ld_dd_mem_nn<M, F: FnMut(&mut pc_state::PcState, u16)-> ()> (clock: &mut clocks::Clock, memory: &mut M, mut reg16: F, pc_state: &mut pc_state::PcState) -> () 
    where M: memory::MemoryRW {

    reg16(pc_state, memory.read16(memory.read16(pc_state.get_pc()+2)));

    pc_state.increment_pc(4);
    clock.increment(20);
}

// LD (IX+d), n; LD (IY+d), n
pub fn ld_i_d_n<M, R16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, i16_reg: &mut R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW {

    let tmp16 = i16_reg.get().wrapping_add((memory.read(pc_reg.get()+2) as i8) as u16);
    memory.write(tmp16, memory.read(pc_reg.get()+3));
    pc_state::PcState::increment_reg(pc_reg, 4);
    clock.increment(19);
}

// LD r, (IY+d); LD r, (IY+d);
// 0xDD, 0b01rrr110
// 0xFD, 0b01rrr110
pub fn ld_r_i_d<M, F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState, i16_value: u16, mut dst_fn: F) -> () 
    where M: memory::MemoryRW {

    dst_fn(pc_state, memory.read(i16_value.wrapping_add((memory.read(pc_state.get_pc()+2) as i8) as u16)));
    pc_state.increment_pc(3);
    clock.increment(16);
}

// LD A, R
pub fn ld_a_r(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    // Treat 'r' as relatively random (just connect to cycles) in the lower 7 bits.  Keep the highest bit.
    pc_state.set_r(((clock.cycles >> 2) & 0x7F) as u8 | (pc_state.get_r() & 0x80));
    pc_state.set_a(pc_state.get_r());
    let mut f_status = pc_state.get_f();
    status_flags::accumulator_flags(&mut f_status, pc_state.get_a(), pc_state.get_iff2());
    pc_state.set_f(f_status);

    pc_state.increment_pc(2);
    clock.increment(9);
}

// LD A, I
pub fn ld_a_i(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    pc_state.set_a(pc_state.get_i());
    let mut f_status = pc_state.get_f();
    status_flags::accumulator_flags(&mut f_status, pc_state.get_a(), pc_state.get_iff2());
    pc_state.set_f(f_status);

    pc_state.increment_pc(2);
    clock.increment(9);
}

// LD R, A
pub fn ld_r_a(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    pc_state.set_r(pc_state.get_a());
    pc_state.increment_pc(2);
    clock.increment(9);
}

// LD I, A
pub fn ld_i_a(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) -> () {
    pc_state.set_i(pc_state.get_a());
    pc_state.increment_pc(2);
    clock.increment(9);
}

///////////////////////////////////////////////////////////////////////
//  BIT instructions
///////////////////////////////////////////////////////////////////////

// BIT b, r
pub fn bit_b_r<R16, F16>(clock: &mut clocks::Clock, bit_pos: u8,  r: u8, pc_reg: &mut R16, af_reg: &mut F16) -> () 
    where R16: pc_state::Reg16RW, F16: pc_state::FlagReg  {

    let mut f_status = af_reg.get_flags();
    status_flags::set_bit_test_flags(r, bit_pos, &mut f_status);
    af_reg.set_flags(&f_status);
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(8);
}

// BIT b, (HL) 
pub fn bit_b_mem<M, R16, F16>(clock: &mut clocks::Clock, memory: &mut M, bit_pos: u8, pc_reg: &mut R16, af_reg: &mut F16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW, F16: pc_state::FlagReg  {

    let mut f_status = af_reg.get_flags();
    status_flags::set_bit_test_flags(memory.read(addr_reg.get()), bit_pos, &mut f_status);
    af_reg.set_flags(&f_status);
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(12);
}

pub fn set_b_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, bit_pos:u8, pc_state: &mut pc_state::PcState, mut dst_fn: F, original_value: u8) -> () {
    dst_fn(pc_state, original_value | (0x1 << bit_pos));
    pc_state.increment_pc(2);
    clock.increment(8);
}

// SET b, (HL) 
pub fn set_b_mem<M, R16>(clock: &mut clocks::Clock, memory: &mut M, bit_pos: u8, pc_reg: &mut R16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW {

    memory.write(addr_reg.get(), memory.read(addr_reg.get()) | (0x1 << bit_pos));

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(12);
}

// RES b, r
pub fn res_b_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, bit_pos:u8, pc_state: &mut pc_state::PcState, mut dst_fn: F, original_value: u8) -> () {
    dst_fn(pc_state, original_value & !(0x1 << bit_pos));
    pc_state.increment_pc(2);
    clock.increment(8);
}

// RES b, (HL) 
pub fn res_b_mem<M, R16>(clock: &mut clocks::Clock, memory: &mut M, bit_pos: u8, pc_reg: &mut R16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW {

    memory.write(addr_reg.get(), memory.read(addr_reg.get()) & !(0x1 << bit_pos));

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(12);
}

// BIT b, (IY+d),  BIT b, (IX+d) (if mem at pc + 3 -> 0b01XXXXXX)
// RES b, (IY+d),  RES b, (IX+d) (if mem at pc + 3 -> 0b10XXXXXX)
// SET b, (IY+d),  SET b, (IX+d) (if mem at pc + 3 -> 0b11XXXXXX)
// (if mem at pc + 3 -> 0b11XXXXXX) -> ERROR
pub fn bit_res_set_b_i_d<M, R16, F16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, af_reg: &mut F16, i16_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW, F16: pc_state::FlagReg  {

    let tmp16 = i16_reg.get().wrapping_add((memory.read(pc_reg.get()+2) as i8) as u16);
    let test_value  = memory.read(tmp16);
    let op_details    = memory.read(pc_reg.get()+3);
    let bit_pos = (op_details >> 3) & 0x7;

    match op_details >> 6 {
        0b01 => {
                    /* BIT b */
                    let mut f_status = af_reg.get_flags();
                    status_flags::set_bit_test_flags(test_value, bit_pos, &mut f_status);
                    af_reg.set_flags(&f_status);
                },
        0b10 => {
                    /* RES b */
                    memory.write(tmp16, test_value & !(0x1 << bit_pos));
                }
        0b11 => {
                    /* SET b */
                    memory.write(tmp16, test_value | (0x1 << bit_pos));
                }
        _ => {panic!("Unsupported byte value! {}", op_details);}
    }

    pc_state::PcState::increment_reg(pc_reg, 4);
    clock.increment(23);
}

///////////////////////////////////////////////////////////////////////
//  Jump instructions
///////////////////////////////////////////////////////////////////////

//  JP (IX), JP (IY)
// Load PC with IX, IY, to jump to that location.
pub fn jp_i<R16>(clock: &mut clocks::Clock, pc_reg: &mut R16, i16_reg: &R16) -> () where R16: pc_state::Reg16RW {
    pc_reg.set(i16_reg.get()); 
    clock.increment(8);
}

// CP n
// Compare accumulator with 'n' to set status flags (but don't change accumulator)
pub fn cp_i_d<M>(clock: &mut clocks::Clock, memory: &mut M, i16_value: u16, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {

    // This function sets the 'pc_state.f'
    instruction_set::cp_flags(pc_state.get_a(),  memory.read(i16_value.wrapping_add((memory.read(pc_state.get_pc()+2) as i8) as u16)), &mut pc_state.af_reg);

    pc_state.increment_pc(3);
    clock.increment(19);
}

////////////////////////////////////////////////////
// 16-bit arithmetic Group
////////////////////////////////////////////////////

// ADD HL, ss
pub fn add16<R16, F16>(clock: &mut clocks::Clock, src_value: u16, 
             pc_reg: &mut R16, dst_reg: &mut R16, af_reg: &mut F16) -> () where R16: pc_state::Reg16RW, F16: pc_state::FlagReg {

    dst_reg.set(instruction_set::add16c(dst_reg.get(), src_value, false, af_reg));

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(15);
}

////////////////////////////////////////////////////
// Rotate and shift group
////////////////////////////////////////////////////

// RL r
// Rotate Left 
fn common_rotate_shift<F: FnMut(&mut pc_state::PcState, u8)-> (), Rot: Fn(u8, bool)->(u8, bool) >(shift_rot_fn: Rot, clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, mut dst_fn: F, src: u8) -> () {
    let mut f_value = pc_state.get_f();
    let (new_value, carry) =  shift_rot_fn(src, f_value.get_c()==1);

    dst_fn(pc_state, new_value);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    pc_state.set_f(f_value);

    pc_state.increment_pc(2);
    clock.increment(8);
}

// RRC r
// Rotate Right with carry
pub fn rrc_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    // Create closure for unused argument
    common_rotate_shift(|input, _carry|{instruction_set::rotate_right_carry(input)}, clock, pc_state, dst_fn, src);
}

// RR r
// Rotate Right 
pub fn rr_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    common_rotate_shift(instruction_set::rotate_right, clock, pc_state, dst_fn, src);
}

// RLC r
// Rotate Left with carry
pub fn rlc_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    // Create closure for unused argument
    common_rotate_shift(|input, _carry|{instruction_set::rotate_left_carry(input)}, clock, pc_state, dst_fn, src);
}

// RL r
// Rotate Left 
pub fn rl_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    common_rotate_shift(instruction_set::rotate_left, clock, pc_state, dst_fn, src);
}

// SLA r
// Shift Left Arithmetic
pub fn sla_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    // Create closure for unused argument
    common_rotate_shift(|input, _carry|{instruction_set::shift_left_arithmetic(input)}, clock, pc_state, dst_fn, src);
}

// SLL r
// Shift Left Logical (?) undocumented, inserts a 1 in the lower bit
pub fn sll_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    // Create closure for unused argument
    common_rotate_shift(|input, _carry|{instruction_set::shift_left_logical(input)}, clock, pc_state, dst_fn, src);
}


// SRA r
// Shift Right Arithmetic
pub fn sra_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    // Create closure for unused argument
    common_rotate_shift(|input, _carry|{instruction_set::shift_right_arithmetic(input)}, clock, pc_state, dst_fn, src);
}

// SRL r
// Shift Right Logical
pub fn srl_r<F: FnMut(&mut pc_state::PcState, u8)-> ()>(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState, dst_fn: F, src: u8) -> () {
    // Create closure for unused argument
    common_rotate_shift(|input, _carry|{instruction_set::shift_right_logical(input)}, clock, pc_state, dst_fn, src);
}

// RLC (HL) 
pub fn rlc_hl<M, R16, F16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, af_reg: &mut F16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW, F16: pc_state::FlagReg 
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::rotate_left_carry(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(15);
}

// RRC (HL) 
pub fn rrc_hl<M, R16, F16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, af_reg: &mut F16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW, F16: pc_state::FlagReg 
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::rotate_right_carry(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(15);
}

// SLA (HL) 
pub fn sla_hl<M, R16, F16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, af_reg: &mut F16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW, F16: pc_state::FlagReg 
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::shift_left_arithmetic(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(15);
}


// SRA (HL) 
pub fn sra_hl<M, R16, F16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, af_reg: &mut F16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW, F16: pc_state::FlagReg 
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::shift_right_arithmetic(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(15);
}

// SRL (HL) 
pub fn srl_hl<M, R16, F16>(clock: &mut clocks::Clock, memory: &mut M, pc_reg: &mut R16, af_reg: &mut F16, addr_reg: & R16) -> () 
    where M: memory::MemoryRW, R16: pc_state::Reg16RW, F16: pc_state::FlagReg 
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::shift_right_logical(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(15);
}



// # Addition instructions
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
