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

/*************************************************************************************/
/* Extended Load Instructions                                                        */
/*************************************************************************************/

// LD (IX+d), r, LD (IY+d), 
//
// pub fn ld_iy_d_r
// pub fn ld_ix_d_r
pub fn ld_i_d_r(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, r: u8, pc_reg: &mut dyn pc_state::Reg16RW, i16_reg: &dyn pc_state::Reg16RW) -> () {
    memory.write(i16_reg.get().wrapping_add((memory.read(pc_reg.get()+2) as i8) as u16), r); 
    pc_state::PcState::increment_reg(pc_reg, 3);
    clock.increment(19);
}


// LD I, nn
// LD IX, nn; LD IY, nn
pub fn ld_i_nn(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, r: u8, pc_reg: &mut dyn pc_state::Reg16RW, i16_reg: &mut dyn pc_state::Reg16RW) -> () {
    i16_reg.set(memory.read16(pc_reg.get() + 2));
    pc_state::PcState::increment_reg(pc_reg, 4);
    clock.increment(20);
}

// LD I, (nn)
// LD IX, (nn); LD IY, (nn)
// was ld_i__nn
pub fn ld_i_mem_nn(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, r: u8, pc_reg: &mut dyn pc_state::Reg16RW, i16_reg: &mut dyn pc_state::Reg16RW) -> () {
    i16_reg.set(memory.read16(memory.read16(pc_reg.get()+2)));
    pc_state::PcState::increment_reg(pc_reg, 4);
    clock.increment(20);
}

// BIT b, r
pub fn bit_b_r(clock: &mut clocks::Clock, bit_pos: u8,  r: u8, pc_reg: &mut dyn pc_state::Reg16RW, af_reg: &mut pc_state::FlagReg16) -> () {
    let mut f_status = af_reg.get_flags();
    status_flags::set_bit_test_flags(r, bit_pos, &mut f_status);
    af_reg.set_flags(&f_status);
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(8);
}

// BIT b, (HL) 
pub fn bit_b_mem(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, bit_pos: u8, pc_reg: &mut dyn pc_state::Reg16RW, af_reg: &mut pc_state::FlagReg16, addr_reg: & dyn pc_state::Reg16RW) -> () {
    let mut f_status = af_reg.get_flags();
    status_flags::set_bit_test_flags(memory.read(addr_reg.get()), bit_pos, &mut f_status);
    af_reg.set_flags(&f_status);
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(12);
}

// BIT b, (IY+d),  BIT b, (IX+d) (if mem at pc + 3 -> 0b01XXXXXX)
// RES b, (IY+d),  RES b, (IX+d) (if mem at pc + 3 -> 0b10XXXXXX)
// SET b, (IY+d),  SET b, (IX+d) (if mem at pc + 3 -> 0b11XXXXXX)
// (if mem at pc + 3 -> 0b11XXXXXX) -> ERROR
pub fn bit_res_set_b_i_d(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, pc_reg: &mut dyn pc_state::Reg16RW, af_reg: &mut pc_state::FlagReg16, i16_reg: & dyn pc_state::Reg16RW) -> () {
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

pub fn bit_b_i_d(clock: &mut clocks::Clock, memory: &mut memory::MemoryAbsolute, bit_pos: u8, pc_reg: &mut dyn pc_state::Reg16RW, af_reg: &mut pc_state::FlagReg16, i16_reg: & dyn pc_state::Reg16RW) -> () {
//    let mut f_status = af_reg.get_flags();
//    status_flags::set_bit_test_flags(memory.read(addr_reg.get()), bit_pos, &mut f_status);
//    pc_state::PcState::increment_reg(pc_reg, 2);
//    clock.increment(12);
}


// # Addition instructions
// 
// # Exists a both 'normal' and 'extended'
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
