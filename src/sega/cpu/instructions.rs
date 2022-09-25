use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interruptor;
use super::super::ports;
use super::instruction_set;
use super::extended_instruction_set;

pub struct Instruction {
}

// Gets the value from the particular 8-bit register.
fn select_8_bit_read_register (pc_state: &pc_state::PcState, reg_select: u8) -> u8 {
    let src = match reg_select & 0x7 {
        0 => {pc_state.get_b()}
        1 => {pc_state.get_c()}
        2 => {pc_state.get_d()}
        3 => {pc_state.get_e()}
        4 => {pc_state.get_h()}
        5 => {pc_state.get_l()}
        7 => {pc_state.get_a()}
        _ => {panic!("Code path that was thought to be unreachable was reached! {}", reg_select);}
    };
    src
}

fn get_condition_result(pc_state: &mut pc_state::PcState, condition_select: u8) -> bool {
    let condition = match condition_select & 0b111 {
        0b000 => {pc_state.get_f().get_z() == 0}  // Non-Zero (NZ)     Z
        0b001 => {pc_state.get_f().get_z() == 1}  // Zero (Z)          Z
        0b010 => {pc_state.get_f().get_c() == 0}  // No Carry (NC)     C
        0b011 => {pc_state.get_f().get_c() == 1}  // Carry (C)         C
        0b100 => {pc_state.get_f().get_pv() == 0} // Parity Odd (PO)   P/V
        0b101 => {pc_state.get_f().get_pv() == 1} // Parity Even (PE)  P/V
        0b110 => {pc_state.get_f().get_s() == 0}  // Sign Positive (P) S
        0b111 => {pc_state.get_f().get_s() == 1}  // Sign Negative (M) S
        _ => {panic!("Code path that was thought to be unreachable was reached! {}", condition_select);}
    };
    condition
}

fn get_8_bit_register_set_function (reg_select: u8) -> impl FnMut(&mut pc_state::PcState, u8) -> () {
    // Return a closure here so as to not borrow pc_state more than once to feed to function.
    // Allows register specific 'set' calls to be selected based on op-code.
    // instruction implementation then calls: fn(pc_state, new_value) to set the register value.
    let dst = move |state: &mut pc_state::PcState, x| match (reg_select) & 0x7 {
            0 => {state.set_b(x)}
            1 => {state.set_c(x)}
            2 => {state.set_d(x)}
            3 => {state.set_e(x)}
            4 => {state.set_h(x)}
            5 => {state.set_l(x)}
            7 => {state.set_a(x)}
            _ => {panic!("Code path that was thought to be unreachable was reached! {}", reg_select);}
        }; 
    dst
}

// Gets the value from the particular 8-bit register.
fn select_16_bit_read_register (pc_state: &pc_state::PcState, reg_select: u8) -> u16 {
    let src = match reg_select & 0x3 {
        0b00 => {pc_state.get_bc()}
        0b01 => {pc_state.get_de()}
        0b10 => {pc_state.get_hl()}
        0b11 => {pc_state.get_af()}
        _ => {panic!("Code path that was thought to be unreachable was reached! {}", reg_select);}
    };
    src
}

// Gets the value from the particular 8-bit register.
fn get_16_bit_ss_set_function(reg_select: u8) -> impl FnMut(&mut pc_state::PcState, u16) -> () {
    let reg16 = move |state: &mut pc_state::PcState, x| match (reg_select) & 0x3 {
        0b00 => {state.set_bc(x)}
        0b01 => {state.set_de(x)}
        0b10 => {state.set_hl(x)}
        0b11 => {state.set_af(x)}
        _ => {panic!("Code path that was thought to be unreachable was reached! {}", reg_select);}
    };
    reg16
}


impl Instruction {
    pub fn execute<M>(op_code: u8, clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interruptor: &mut interruptor::Interruptor) -> () where M: memory::MemoryRW{
        match op_code {
            // Extended op codes, not executed directly
            0xcb => { Self::execute_cb(clock, memory, pc_state, interruptor);}
            0xdd => { Self::execute_dd(clock, memory, pc_state, interruptor);}
            0xed => { Self::execute_ed(clock, memory, pc_state, ports, interruptor);}
            0xfd => { Self::execute_fd(clock, memory, pc_state, interruptor);}

            0xfb => { 
                pc_state.increment_pc(1);
                // Perform a 'step' before enabling interrupts.
                let next_op_code = memory.read(pc_state.get_pc());
                Self::execute(next_op_code, clock, memory, pc_state, ports, interruptor);

                instruction_set::ei(clock, pc_state);
                // TODO: Actually do the polling call
                //  if (self.poll_interupts(self.clocks.cycles) == True):
                      interruptor::Interruptor::interrupt(pc_state, memory);
            }

            0x00 => { instruction_set::noop(clock, pc_state);
            }
            0x01 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.bc_reg);} // LD dd, nn : for BC
            0x02 => { instruction_set::ld_mem_r(clock, memory, pc_state.get_a(), &mut pc_state.pc_reg, &mut pc_state.bc_reg);} // LD (BC), A
            0x12 => { instruction_set::ld_mem_r(clock, memory, pc_state.get_a(), &mut pc_state.pc_reg, &mut pc_state.de_reg);} // LD (DE), A

            n if (n & 0b11001111 == 0b00001001) => {
                let ss = (n >> 4) & 0x3;
                instruction_set::add16(clock, select_16_bit_read_register(pc_state, ss), 
                                       &mut pc_state.pc_reg, &mut pc_state.hl_reg, &mut pc_state.af_reg);
            }

            0x0f => { instruction_set::rrca(clock, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.get_a());}
            0x1f => { instruction_set::rra(clock, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.get_a());}
            0x07 => { instruction_set::rlca(clock, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.get_a());}
            0x17 => { instruction_set::rla(clock, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.get_a());}

            0x10 => { instruction_set::djnz(clock, memory, pc_state);} // DJNZ n
            0x11 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.de_reg);} // LD DE, nn
            0x21 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD HL, nn
            0x2a => { instruction_set::ld_r16_mem(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD HL, (nn)
            0x31 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg);} // LD DE, nn
            0x20 => { instruction_set::jrnz_e(clock, memory, pc_state);
            } // JR NZ, e
            0x28 => { instruction_set::jrz_e(clock, memory, pc_state);} // JR Z, e

            // INC ss,  Op Code: 0b00ss0011
            n if (n & 0b11001111 == 0b00000011) => {
                let ss = (n >> 4) & 0x3;
                instruction_set::inc_16(clock, get_16_bit_ss_set_function(ss), pc_state, select_16_bit_read_register(pc_state, ss));
            }
            
            // DEC ss,  Op Code: 0b00ss1011
            n if (n & 0b11001111 == 0b00001011) => {
                let ss = (n >> 4) & 0x3;
                instruction_set::dec_16(clock, get_16_bit_ss_set_function(ss), pc_state, select_16_bit_read_register(pc_state, ss));
            }

            0x34 => { instruction_set::inc_hl(clock, memory, pc_state);} // INC HL
            0x35 => { instruction_set::dec_hl(clock, memory, pc_state);} // DEC HL
            0x36 => { instruction_set::ld_mem_n(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD (HL), n
            0x3a => { instruction_set::ld_r8_mem(clock, memory, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)});} // LD A, (n)

            // inc_r instructions
            // op_code: 0b00rrr100
            n if (n & 0b11000111 == 0b00000100) && (((n >> 3) & 0x7) != 0x6) => {
                    let reg_index = (op_code >> 3) & 0x7;
                    let dst_fn = get_8_bit_register_set_function (reg_index);
                    instruction_set::inc_r(clock, pc_state, dst_fn, select_8_bit_read_register(pc_state, reg_index));
                }

            // ld_mem_r instructions
            // opcode: 0b01110rrr 
            n if (n & 0b11111000 == 0b01110000) && (n != 0x76) => {
                    let reg_index = n & 0x7;
                    instruction_set::ld_mem_r(clock, memory, 
                            select_8_bit_read_register(pc_state, reg_index), // gets the appropriate register getter fromt the supplied op-code
                            &mut pc_state.pc_reg, &mut pc_state.hl_reg); // LD (HL), r
                }

            // ADD r
            // op code: 0b10000rrr
            n if (n & 0b11111000 == 0b10000000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    instruction_set::add_r(clock, select_8_bit_read_register(pc_state, reg_index), pc_state);
            }

            // SUB r
            // op code: 0b10011rrr
            n if (n & 0b11111000 == 0b10011000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    instruction_set::sub_r(clock, select_8_bit_read_register(pc_state, reg_index), pc_state);
            }

            // AND r
            // op code: 0b10100rrr
            n if (n & 0b11111000 == 0b10100000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    instruction_set::and_r(clock, select_8_bit_read_register(pc_state, reg_index), pc_state);
            }

            // XOR r
            // op code: 0b10101rrr
            n if (n & 0b11111000 == 0b10101000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    instruction_set::xor_r(clock, select_8_bit_read_register(pc_state, reg_index), pc_state);
            }

            // OR r
            // op code: 0b10101rrr
            n if (n & 0b11111000 == 0b10110000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    instruction_set::or_r(clock, select_8_bit_read_register(pc_state, reg_index), pc_state);
            }

            // cp_r instructions
            // opcode: 0b10111rrr 
            n if (n & 0b11111000 == 0b10111000) && (n != 0b11111110) => {
                    let reg_index = n & 0x7;
                    instruction_set::cp_r(clock,
                            select_8_bit_read_register(pc_state, reg_index), // gets the appropriate register getter fromt the supplied op-code
                            pc_state); // CP r
                }

            // JP cc, nn instructions
            // opcode: 0b11ccc010 
            n if (n & 0b11000111 == 0b11000010) => {
                    let condition_select = (n >> 3) & 0b111;
                    let condition = get_condition_result(pc_state, condition_select);
                    instruction_set::jump_cc_nn(clock, memory, pc_state, condition);
                }

            0xd3 => { instruction_set::out_n_a(clock, memory, pc_state, ports);} // OUT (n), cpu_state->A
            0xd9 => { instruction_set::exx(clock, pc_state);} // EXX
            0xe6 => { instruction_set::and_n(clock, memory, pc_state);} // AND n
            0xfe => { instruction_set::cp_n(clock, memory, pc_state);} // CP n

            // dec_r instructions
            // op_code: 0b00rrr101
            n if (n & 0b11000111 == 0b00000101) && (((n >> 3) & 0x7) != 0x6) => {
                    let reg_index = (op_code >> 3) & 0x7;
                    let dst_fn = get_8_bit_register_set_function (reg_index);
                    instruction_set::dec_r(clock, pc_state, dst_fn, select_8_bit_read_register(pc_state, reg_index));
                }

            // ld_r_mem instructions (eg // LD r, (HL)
            // op_code: 0b01rrr110  // LD r, (HL)
            n if (n & 0b11000111 == 0b01000110) && (((n >> 3) & 0x7) != 0x6) => {
                    let reg_index = (op_code >> 3) & 0x7;
                    let dst_fn = get_8_bit_register_set_function (reg_index);
                    instruction_set::ld_r_mem(clock, memory, pc_state, dst_fn, pc_state.hl_reg.get()); // LD r, (HL)
                }
            0x0a => { instruction_set::ld_r_mem(clock, memory, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.bc_reg.get());} // LD A, (BC)
            0x1a => { instruction_set::ld_r_mem(clock, memory, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.de_reg.get());} // LD A, (DE)

            // LD r,n
            // opcode: 0b00rrr110 nnnnnnnn
            n if (n & 0b11000111 == 0b00000110) && (((n >> 3) & 0x7) != 0x6) => {
                    let reg_index = (op_code >> 3) & 0x7;
                    let dst_fn = get_8_bit_register_set_function (reg_index);
                    instruction_set::ld_r(clock, memory, pc_state, dst_fn); // LD r, n
                }

            // ld_r_r instructions ( 0b01dddsss) 
            n if ((n & 0b11000000) == 0b01000000) && ((n & 0x07) != 0x6) && ((n & 0x38) != 0x3) => {
                    // gets the appropriate register getter fromt the supplied op-code
                    let dst_reg_index = (n >> 3) & 0x7;
                    let src_reg_index = n & 0x7;
                    let dst_fn = get_8_bit_register_set_function (dst_reg_index);
                    instruction_set::ld_r_r(clock, 
                            select_8_bit_read_register(pc_state, src_reg_index), 
                            pc_state, dst_fn);
                }
            0xc9 => { instruction_set::ret(clock, memory, pc_state);} // RET
            0x08 => { instruction_set::ex(clock, &mut pc_state.pc_reg, &mut pc_state.af_reg, &mut pc_state.shadow_af_reg);}
            0x18 => { instruction_set::jr_e(clock, memory, pc_state);}
             
            0x22 => { instruction_set::ld_mem_nn_hl(clock, memory, pc_state);}
            0x27 => { instruction_set::daa(clock, pc_state);}
            0x2f => { instruction_set::cpl(clock, pc_state);}
            0x30 => { instruction_set::jrnc_e(clock, memory, pc_state);}
            0x32 => { instruction_set::ld_nn_r(clock, memory, pc_state.get_a(), &mut pc_state.pc_reg);}
            0x37 => { instruction_set::scf(clock, &mut pc_state.pc_reg, &mut pc_state.af_reg);}
            0x38 => { instruction_set::jrc_e(clock, memory, pc_state);}
            0x3f => { instruction_set::ccf(clock, &mut pc_state.pc_reg, &mut pc_state.af_reg);}
            0x76 => { instruction_set::halt(clock);}
            0x86 => { instruction_set::add_hl(clock, memory, pc_state);}

            // ADC r
            // op code: 0b10001rrr
            n if (n & 0b11111000 == 0b10001000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    instruction_set::adc_r(clock, select_8_bit_read_register(pc_state, reg_index), pc_state);
            }

            // SBC r
            // op code: 0b10011rrr
            n if (n & 0b11111000 == 0b10011000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    instruction_set::sbc_r(clock, select_8_bit_read_register(pc_state, reg_index), pc_state);
            }

            // CALL cc, nn instructions
            // opcode: 0b11ccc100 
            n if (n & 0b11000111 == 0b11000100) => {
                    let condition_select = (n >> 3) & 0b111;
                    let condition = get_condition_result(pc_state, condition_select);
                    instruction_set::call_cc_nn(clock, memory, pc_state, condition);
                }
            0xcd => { instruction_set::call_nn(clock, memory, pc_state);}

            // RET cc instructions
            // opcode: 0b11ccc000 
            n if (n & 0b11000111 == 0b11000000) => {
                    let condition_select = (n >> 3) & 0b111;
                    let condition = get_condition_result(pc_state, condition_select);
                    instruction_set::ret_cc(clock, memory, pc_state, condition);
            }
            0x8e => { instruction_set::adc_hl(clock, memory, pc_state);}
            0x96 => { instruction_set::sub_hl(clock, memory, pc_state);}
            0x9e => { instruction_set::sbc_hl(clock, memory, pc_state);}
            0xa6 => { instruction_set::and_hl(clock, memory, pc_state);}
            0xae => { instruction_set::xor_hl(clock, memory, pc_state);}
            0xb6 => { instruction_set::or_hl(clock, memory, pc_state);}
            0xbe => { instruction_set::cp_hl(clock, memory, pc_state);}
            0xc1 => { instruction_set::pop(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.bc_reg);}
            0xc3 => { instruction_set::jp_nn(clock, memory, pc_state);}
            0xc5 => { instruction_set::push(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &pc_state.bc_reg);}
            0xc6 => { instruction_set::add_n(clock, memory, pc_state);}
            0xce => { instruction_set::adc_n(clock, memory, pc_state);}
            0xd1 => { instruction_set::pop(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.de_reg);}
            0xd5 => { instruction_set::push(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &pc_state.de_reg);}
            0xd6 => { instruction_set::sub_n(clock, memory, pc_state);}
            0xdb => { instruction_set::in_a_n(clock, memory, pc_state, ports);}
            0xde => { instruction_set::sbc_n(clock, memory, pc_state);}
            0xe1 => { instruction_set::pop(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.hl_reg);}
            0xe3 => { instruction_set::ex_sp_hl(clock, memory, pc_state);}
            0xe5 => { instruction_set::push(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &pc_state.hl_reg);}
            0xe9 => { instruction_set::jp_hl(clock, &pc_state.hl_reg, &mut pc_state.pc_reg);}
            0xeb => { instruction_set::ex(clock, &mut pc_state.pc_reg, &mut pc_state.de_reg, &mut pc_state.shadow_hl_reg);}
            0xee => { instruction_set::xor_n(clock, memory, pc_state);}
            0xf1 => { instruction_set::pop(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.af_reg);}
            0xf3 => { instruction_set::di(clock, pc_state);}
            0xf5 => { instruction_set::push(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &pc_state.af_reg);}
            0xf6 => { instruction_set::or_n(clock, memory, pc_state);}
            0xf9 => { instruction_set::ld_sp_hl(clock, &pc_state.hl_reg, &mut pc_state.pc_reg, &mut pc_state.sp_reg);}

            // rst instructions
            // opcode: 0b11ttt111 -> ttt -> (0x0, 0x8, 0x10, 0x18, 0x20, 0x30, 0x38)
            n if (n & 0b11000111 == 0b11000000) => {
                let rst_addr = n & 0x38;
                instruction_set::rst(clock, memory, pc_state, rst_addr);
            }
        
            _ => {panic!("Opcode not implemented: {:x}", op_code); }

        }
    } 

    // Extended instructions
    pub fn execute_cb<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           interruptor: &mut interruptor::Interruptor) -> () where M: memory::MemoryRW {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            // BIT b, r
            // 0xCB, 0b01bbbrrr
            n if (n & 0b11000000 == 0b01000000) && (n & 0b111 != 0b110) => {
                    let bit_pos = (n >> 3) & 0x7;
                    let reg_index = n & 0x7;
                    let r = select_8_bit_read_register(pc_state, reg_index);
                    extended_instruction_set::bit_b_r(clock, bit_pos, r, &mut pc_state.pc_reg, &mut pc_state.af_reg);
                }
            n if (n & 0b11000110 == 0b01000110) => {
                    let bit_pos = (n >> 3) & 0x7;
                    extended_instruction_set::bit_b_mem(clock, memory, bit_pos, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.hl_reg);
                }

            // SET b, r
            // 0xCB, 0b11bbbrrr
            n if (n & 0b11000000 == 0b11000000) && (n & 0b111 != 0b110) => {
                    let bit_pos = (n >> 3) & 0x7;
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::set_b_r(clock, bit_pos, pc_state, dst_fn, current_r);
                }
            n if (n & 0b11000110 == 0b11000110) => {
                    let bit_pos = (n >> 3) & 0x7;
                    extended_instruction_set::set_b_mem(clock, memory, bit_pos, &mut pc_state.pc_reg, &pc_state.hl_reg);
                }

            // SET b, r
            // 0xCB, 0b10bbbrrr
            n if (n & 0b11000000 == 0b10000000) && (n & 0b111 != 0b110) => {
                    let bit_pos = (n >> 3) & 0x7;
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::res_b_r(clock, bit_pos, pc_state, dst_fn, current_r);
                }
            n if (n & 0b11000110 == 0b10000110) => {
                    let bit_pos = (n >> 3) & 0x7;
                    extended_instruction_set::res_b_mem(clock, memory, bit_pos, &mut pc_state.pc_reg, &pc_state.hl_reg);
                }

            // RLC r
            // 0xCB, 0b00000rrr
            n if (n & 0b11111000 == 0b00000000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::rlc_r(clock, pc_state, dst_fn, current_r);
                }

            // RRC r
            // 0xCB, 0b00001rrr
            n if (n & 0b11111000 == 0b00001000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::rrc_r(clock, pc_state, dst_fn, current_r);
                }

            // RL r
            // 0xCB, 0b00010rrr
            n if (n & 0b11111000 == 0b00010000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::rl_r(clock, pc_state, dst_fn, current_r);
                }

            // RR r
            // 0xCB, 0b00011rrr
            n if (n & 0b11111000 == 0b00011000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::rr_r(clock, pc_state, dst_fn, current_r);
                }

            // SLA r
            // 0xCB, 0b00100rrr
            n if (n & 0b11111000 == 0b00100000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::sla_r(clock, pc_state, dst_fn, current_r);
                }

            // SRA r
            // 0xCB, 0b00101rrr
            n if (n & 0b11111000 == 0b00101000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::sra_r(clock, pc_state, dst_fn, current_r);
                }

            // SLL r
            // 0xCB, 0b00110rrr
            n if (n & 0b11111000 == 0b00110000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::sll_r(clock, pc_state, dst_fn, current_r);
                }

            // SRL r
            // 0xCB, 0b00111rrr
            n if (n & 0b11111000 == 0b00111000) && (n & 0b111 != 0b110) => {
                    let reg_index = op_code & 0x7;
                    let current_r = select_8_bit_read_register(pc_state, reg_index);
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::srl_r(clock, pc_state, dst_fn, current_r);
                }
          0x06 => { extended_instruction_set::rlc_hl(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.hl_reg);}
          0x0e => { extended_instruction_set::rrc_hl(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.hl_reg);}
          0x26 => { extended_instruction_set::sla_hl(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.hl_reg);}
          0x2e => { extended_instruction_set::sra_hl(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.hl_reg);}
          0x36 => { extended_instruction_set::srl_hl(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.hl_reg);}

            _ => {panic!("Extended(0xCB) Opcode not implemented: {:x}", op_code); }

        }
    } 

    // Extended instructions
    pub fn execute_dd<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           interruptor: &mut interruptor::Interruptor) -> () where M: memory::MemoryRW {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0xcb => { extended_instruction_set::bit_res_set_b_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.ix_reg);}
            0x22 => { extended_instruction_set::ld_mem_nn_reg16(clock, memory, &mut pc_state.pc_reg, &pc_state.ix_reg);}
            0x2A => { extended_instruction_set::ld_i_mem_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.ix_reg);}
            0x36 => { extended_instruction_set::ld_i_d_n(clock, memory, &mut pc_state.pc_reg, &mut pc_state.ix_reg);}

            n if (n & 0b11000111 == 0b01000110) && ((n >> 3) & 0b111 != 0b110) => {
                    let reg_index = (n >> 3) & 0x7;
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::ld_r_i_d(clock, memory, pc_state, pc_state.ix_reg.get(), dst_fn);
                }

            // LD (IX+d)
            // op code:  0xDD, 0b01110rrr, 0bdddddddd
            n if (n & 0b11111000 == 0b01110000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    extended_instruction_set::ld_i_d_r(clock, memory, select_8_bit_read_register(pc_state, reg_index), &mut pc_state.pc_reg, &pc_state.ix_reg);
            }

            0xE9 => { extended_instruction_set::jp_i(clock, &mut pc_state.pc_reg, &pc_state.ix_reg);}
            0x21 => { extended_instruction_set::ld_i_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.ix_reg);}
            0xBE => { extended_instruction_set::cp_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.ix_reg, &mut pc_state.af_reg);}

            n if (n & 0b11001111 == 0b00001001) => {
                let ss = (n >> 4) & 0x3;
                extended_instruction_set::add16(clock, select_16_bit_read_register(pc_state, ss), 
                                                &mut pc_state.pc_reg, &mut pc_state.ix_reg, &mut pc_state.af_reg);
            }

            0x23 => { extended_instruction_set::inc_16(clock, &mut pc_state.pc_reg, &mut pc_state.ix_reg);}
            0x2B => { extended_instruction_set::dec_16(clock, &mut pc_state.pc_reg, &mut pc_state.ix_reg);}
            0x34 => { extended_instruction_set::inc_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &mut pc_state.ix_reg);}
            0x35 => { extended_instruction_set::dec_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &mut pc_state.ix_reg);}
            0x8E => { extended_instruction_set::adc_ix_d(clock, memory, pc_state);}
            0x96 => { extended_instruction_set::sub_ix_d(clock, memory, pc_state);}
            0xA6 => { extended_instruction_set::and_ix_d(clock, memory, pc_state);}
            0xAE => { extended_instruction_set::xor_ix_d(clock, memory, pc_state);}
            0xB6 => { extended_instruction_set::or_ix_d(clock, memory, pc_state);}
            0x86 => { extended_instruction_set::add_ix_d(clock, memory, pc_state);}
            0xE1 => { extended_instruction_set::pop_i(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.ix_reg);}
            0xE5 => { extended_instruction_set::push_i(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.ix_reg);}
            0xE3 => { extended_instruction_set::ex_sp_i(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.ix_reg);}

            _ => {panic!("Extended(0xDD) Opcode not implemented: {:x}", op_code); }
        }
    } 
    // Extended instructions
    pub fn execute_fd<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           interruptor: &mut interruptor::Interruptor) -> () where M: memory::MemoryRW {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0xcb => { extended_instruction_set::bit_res_set_b_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &pc_state.iy_reg);}
            0x22 => { extended_instruction_set::ld_mem_nn_reg16(clock, memory, &mut pc_state.pc_reg, &pc_state.iy_reg);}
            0x2A => { extended_instruction_set::ld_i_mem_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.iy_reg);}
            0x36 => { extended_instruction_set::ld_i_d_n(clock, memory, &mut pc_state.pc_reg, &mut pc_state.iy_reg);}

            n if (n & 0b11000111 == 0b01000110) && ((n >> 3) & 0b111 != 0b110) => {
                    let reg_index = (n >> 3) & 0x7;
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::ld_r_i_d(clock, memory, pc_state, pc_state.iy_reg.get(), dst_fn);
                }

            // LD (IY+d)
            // op code:  0xFD, 0b01110rrr, 0bdddddddd
            n if (n & 0b11111000 == 0b01110000) && (n  & 0b111 != 0b110) => {
                    let reg_index = n & 0x7;
                    extended_instruction_set::ld_i_d_r(clock, memory, select_8_bit_read_register(pc_state, reg_index), &mut pc_state.pc_reg, &pc_state.iy_reg);
            }

            0xE9 => { extended_instruction_set::jp_i(clock, &mut pc_state.pc_reg, &pc_state.iy_reg);}
            0x21 => { extended_instruction_set::ld_i_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.iy_reg);}
            0xBE => { extended_instruction_set::cp_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.iy_reg, &mut pc_state.af_reg);}
            n if (n & 0b11001111 == 0b00001001) => {
                let ss = (n >> 4) & 0x3;
                extended_instruction_set::add16(clock, select_16_bit_read_register(pc_state, ss), 
                                                &mut pc_state.pc_reg, &mut pc_state.iy_reg, &mut pc_state.af_reg);
            }

            0x23 => { extended_instruction_set::inc_16(clock, &mut pc_state.pc_reg, &mut pc_state.iy_reg);}
            0x2B => { extended_instruction_set::dec_16(clock, &mut pc_state.pc_reg, &mut pc_state.iy_reg);}
            0x34 => { extended_instruction_set::inc_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &mut pc_state.iy_reg);}
            0x35 => { extended_instruction_set::dec_i_d(clock, memory, &mut pc_state.pc_reg, &mut pc_state.af_reg, &mut pc_state.iy_reg);}
            0x8E => { extended_instruction_set::adc_iy_d(clock, memory, pc_state);}
            0x96 => { extended_instruction_set::sub_iy_d(clock, memory, pc_state);}
            0xA6 => { extended_instruction_set::and_iy_d(clock, memory, pc_state);}
            0xAE => { extended_instruction_set::xor_iy_d(clock, memory, pc_state);}
            0xB6 => { extended_instruction_set::or_iy_d(clock, memory, pc_state);}
            0x86 => { extended_instruction_set::add_iy_d(clock, memory, pc_state);}
            0xE1 => { extended_instruction_set::pop_i(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.iy_reg);}
            0xE5 => { extended_instruction_set::push_i(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.iy_reg);}
            0xE3 => { extended_instruction_set::ex_sp_i(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg, &mut pc_state.iy_reg);}

             _ => {panic!("Extended(0xFD) Opcode not implemented: {:x}", op_code); }
        }
    } 
    // Extended instructions
    pub fn execute_ed<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interruptor: &mut interruptor::Interruptor) -> () where M: memory::MemoryRW {
        let op_code = memory.read(pc_state.get_pc() + 1);
        println!("clock: {}, op_code: {:x}, pc: {}", clock.cycles, op_code, pc_state.get_pc());

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 

            // 0b00dd0001 -> BC 00, DE 01, HL 10, SP 11
            n if (n & 0b11001111 == 0b00000001) => {
                let dd = (n >> 4) & 0x3;
                extended_instruction_set::ld_dd_mem_nn(clock, memory, get_16_bit_ss_set_function(dd), pc_state);
            }

            // 0b00dd0001 -> dd -> BC 00, DE 01, HL 10, SP 11
            0x43 => { extended_instruction_set::ld_mem_nn_reg16(clock, memory, &mut pc_state.pc_reg, &pc_state.bc_reg);}
            0x53 => { extended_instruction_set::ld_mem_nn_reg16(clock, memory, &mut pc_state.pc_reg, &pc_state.de_reg);}
            0x63 => { extended_instruction_set::ld_mem_nn_reg16(clock, memory, &mut pc_state.pc_reg, &pc_state.hl_reg);}
            0x73 => { extended_instruction_set::ld_mem_nn_reg16(clock, memory, &mut pc_state.pc_reg, &pc_state.sp_reg);}

            0x5F => { extended_instruction_set::ld_a_r(clock, pc_state);}
            0x57 => { extended_instruction_set::ld_a_i(clock, pc_state);}
            0x47 => { extended_instruction_set::ld_i_a(clock, pc_state);}
            0x4F => { extended_instruction_set::ld_r_a(clock, pc_state);}

            0x67 => { extended_instruction_set::rrd(clock, memory, pc_state);}
            0x6f => { extended_instruction_set::rld(clock, memory, pc_state);}

            // IN r (C) 0xED, 01rrr000
            n if (n & 0b11000111 == 0b01000000) && ((n >> 3) & 0b111 != 0b110) => {
                    let reg_index = (n >> 3) & 0x7;
                    let dst_fn = get_8_bit_register_set_function(reg_index);
                    extended_instruction_set::in_r(clock, pc_state.get_c(), pc_state, dst_fn, ports);
                }

            // OUT r (C) 0xED, 01rrr001
            n if (n & 0b11000111 == 0b01000001) && ((n >> 3) & 0b111 != 0b110) => {
                    let reg_index = (n >> 3) & 0x7;
                    extended_instruction_set::out_r(clock, pc_state.get_c(), pc_state, select_8_bit_read_register(pc_state, reg_index), ports);
                }

            0xA3 => { extended_instruction_set::outi(clock, memory, pc_state, ports);}
            0xAB => { extended_instruction_set::outd(clock, memory, pc_state, ports);}
            0x44 => { extended_instruction_set::neg(clock, pc_state);}
            0x4D => { extended_instruction_set::reti(clock, memory, pc_state);}

            0xB3 => { extended_instruction_set::otir(clock, memory, pc_state, ports);}
            0xB8 => { extended_instruction_set::lddr(clock, memory, pc_state);}
            0xB0 => { extended_instruction_set::ldir(clock, memory, pc_state);}
            0xA2 => { extended_instruction_set::ini(clock, memory, pc_state, ports);}
            0xA1 => { extended_instruction_set::cpi(clock, memory, pc_state);}
            0xB1 => { extended_instruction_set::cpir(clock, memory, pc_state);}
            0xA0 => { extended_instruction_set::ldi(clock, memory, pc_state);}
            0x56 => { extended_instruction_set::im_1(clock, pc_state);}

            // ADC HL, ss
            // 0b01ss1010
            n if (n & 0b11001010 == 0b01001010) =>  {
                let ss = (n >> 4) & 0x3;
                extended_instruction_set::adc_hl_r16(clock, select_16_bit_read_register(pc_state, ss), &mut pc_state.pc_reg, &mut pc_state.hl_reg, &mut pc_state.af_reg);
            }

            // SBC HL, ss
            // 0b01ss0010
            n if (n & 0b11001010 == 0b01000010) =>  {
                let ss = (n >> 4) & 0x3;
                extended_instruction_set::sbc_hl_r16(clock, select_16_bit_read_register(pc_state, ss), &mut pc_state.pc_reg, &mut pc_state.hl_reg, &mut pc_state.af_reg);
            }

            _ => {panic!("Extended(0xED) Opcode not implemented: {:x}", op_code); }
        }
    } 
}

#[cfg(test)]
mod tests {
    use crate::sega::cpu::instructions;
    use crate::sega::cpu::pc_state;
    use crate::sega::memory::memory;
    use crate::sega::clocks;
    use crate::sega::interruptor;
    use crate::sega::ports;
    use crate::impl_common_memoryrw;

    // Create a 'test memory' class, to allow simple/arbitrary population of memory.
    pub struct TestMemory {
        pub dummy_memory:     Vec<u8>
    }

    pub struct TestCore {
        pub clock:      clocks::Clock,
        pub memory:     TestMemory,
        pub pc_state:   pc_state::PcState,
        pub ports:      ports::Ports,
        pub interruptor: interruptor::Interruptor,
    }
    impl TestCore {
        pub fn new() -> Self {
            Self {
                clock: clocks::Clock::new(),
                memory: TestMemory::new(),
                pc_state: pc_state::PcState::new(),
                ports: ports::Ports::new(),
                interruptor: interruptor::Interruptor::new(),
            }
        }
    }

    impl TestMemory {
        pub fn new () -> Self {
            Self {
                dummy_memory: Vec::new(),
            }
        }

        fn read(&self, address: memory::AddressType) -> u8 {
            self.dummy_memory[address as usize]
        }

        fn write(&mut self, address: memory::AddressType, data: u8) -> () {
            self.dummy_memory[address as usize] = data;
        }
    }

    // Allow the memory to be used as 'MemoryRW'
    impl_common_memoryrw!(TestMemory);

    fn simple_execute(test_core: &mut TestCore, op_code: Vec<u8>) {
        // Reset the PC counter and call execute
        test_core.pc_state.set_pc(0);
        test_core.clock.cycles = 0;
        let initial_op_code = op_code[0];
        test_core.memory.dummy_memory = op_code; // Op-code to test
        instructions::Instruction::execute(initial_op_code, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interruptor);
    }



    #[test]
    fn test_instruction_match_style_check() {

        #[derive(PartialEq)]
        #[derive(Debug)]
        enum Ops {
                Op0x70 = 1,
                Op0x71,
                Op0x72,
                Op0x73,
                Op0x74,
                Op0x75,
                Op0x76,
                Op0x77,
                Unknown,
        }
        fn check_match(value: u8) -> Ops {
            match value {
                n if (n & 0x78 == 0x70) && (n != 0x76) => {
                            match n & 0x7 
                            {
                              0 => {Ops::Op0x70} // 0x70: LD (HL), B
                              1 => {Ops::Op0x71} // 0x71: LD (HL), C
                              2 => {Ops::Op0x72} // 0x72: LD (HL), D
                              3 => {Ops::Op0x73} // 0x73: LD (HL), E
                              4 => {Ops::Op0x74} // 0x74: LD (HL), H
                              5 => {Ops::Op0x75} // 0x75: LD (HL), L
                              7 => {Ops::Op0x77} // 0x77: LD (HL), A
                              _ => {panic!("Code path that was thought to be unreachable was reached! {}", n);}
                            }

                }
                0x76 => {Ops::Op0x76}
                _ => {Ops::Unknown}
            }
        }
        assert_eq!(check_match(0x72), Ops::Op0x72);
        assert_eq!(check_match(0x76), Ops::Op0x76);
        assert_eq!(check_match(0x77), Ops::Op0x77);
    }

    #[test]
    fn test_ld_r_r_functions() {
        let mut test_core = TestCore::new();

        // ld_r_r instructions ( 0b01dddsss) 
        // 111 -> A, 000 -> B, 001 -> C, 
        // 010 -> D, 011 -> E, 100 -> H, 
        // 101 -> L
        
        assert_eq!(test_core.clock.cycles, 0);
        assert_eq!(test_core.pc_state.get_b(), 0);

        test_core.pc_state.set_c(0x42);
        instructions::Instruction::execute(0b01000001, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interruptor); // LD r,'r  C -> B
        assert_eq!(test_core.pc_state.get_b(), 0x42);
        assert_eq!(test_core.clock.cycles, 4);
    }

    #[test]
    fn test_jump_functions() {
        let mut test_core = TestCore::new();

        test_core.pc_state.set_hl(0x4233);
        test_core.pc_state.set_pc(0x2003);
        instructions::Instruction::execute(0xE9, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interruptor); // JP (HL)
        assert_eq!(test_core.pc_state.get_pc(), 0x4233);
    }

    #[test]
    fn test_dec_functions() {
        let mut test_core = TestCore::new();

        test_core.pc_state.set_h(0x80);
        let mut flags = test_core.pc_state.get_f();
        flags.set_c(1);
        test_core.pc_state.set_f(flags);
        instructions::Instruction::execute(0b00100101, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interruptor); // dec_r, for h
        assert_eq!(test_core.pc_state.get_h(), 0x7F);
        assert_eq!(test_core.pc_state.get_f().get_h(), 1);
        assert_eq!(test_core.pc_state.get_f().get_c(), 1);
        assert_eq!(test_core.pc_state.get_f().get_s(), 0);
    }

    #[test]
    fn test_specific_opcodes() {
        let mut test_core = TestCore::new();

        test_core.memory.dummy_memory = vec![0x00];
        instructions::Instruction::execute(0x00, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interruptor); // no-op
        assert_eq!(test_core.pc_state.get_pc(), 0x1);
        assert_eq!(test_core.clock.cycles, 4);

        // LD dd, nn: for BC
        // Reset the PC counter.
        test_core.pc_state.set_pc(0);
        test_core.clock.cycles = 0;
        let test_op_code = 0x01;
        test_core.memory.dummy_memory = vec![test_op_code, 0x10, 0x33]; // Op-code to test
        instructions::Instruction::execute(test_op_code, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interruptor);
        assert_eq!(test_core.pc_state.get_pc(), 0x3);
        assert_eq!(test_core.pc_state.get_bc(), 0x3310);
        assert_eq!(test_core.clock.cycles, 10);
    }

    #[test]
    fn test_opcode_cycle_times() {
        fn test_op_code_cycle_count(test_core: &mut TestCore, op_code: Vec<u8>, expected_pc: u16, expected_cycles: u32) {
            simple_execute(test_core, op_code);
            assert_eq!(test_core.pc_state.get_pc(), expected_pc);
            assert_eq!(test_core.clock.cycles, expected_cycles);
        }

        let mut test_core = TestCore::new();

        test_op_code_cycle_count(&mut test_core, vec![0x00], 1, 4); // no-op
        test_op_code_cycle_count(&mut test_core, vec![0x01, 0x10, 0x33], 3, 10); // LD dd, nn
    }

    #[test]
    fn test_rotate_opcode_cycle_times() {
        fn test_op_code_cycle_count(test_core: &mut TestCore, op_code: Vec<u8>, expected_pc: u16, expected_cycles: u32) {
            simple_execute(test_core, op_code);
            assert_eq!(test_core.pc_state.get_pc(), expected_pc);
            assert_eq!(test_core.clock.cycles, expected_cycles);
        }

        let mut test_core = TestCore::new();

        test_core.pc_state.set_a(0xAB);
        test_core.pc_state.set_hl(0x02);
        test_op_code_cycle_count(&mut test_core, vec![0xED, 0x67, 0xCD], 2, 18); // RRD
        assert_eq!(test_core.memory.dummy_memory[2], 0xBC);
        assert_eq!(test_core.pc_state.get_a(), 0xAD);

        test_core.pc_state.set_a(0xAB);
        test_core.pc_state.set_hl(0x02);
        test_op_code_cycle_count(&mut test_core, vec![0xED, 0x6F, 0xCD], 2, 18); // RLD
        assert_eq!(test_core.pc_state.get_a(), 0xAC);
        assert_eq!(test_core.memory.dummy_memory[2], 0xDB);

        test_core.pc_state.set_ix(0x04);
        test_op_code_cycle_count(&mut test_core, vec![0xDD, 0x34, 0xFF,0x09], 3, 23); // INC (IX+d) (d = -1)
        assert_eq!(test_core.memory.dummy_memory[3], 0xA);

        test_core.pc_state.set_iy(0x06);
        test_op_code_cycle_count(&mut test_core, vec![0xFD, 0x35, 0xFD,0x09], 3, 23); // DEC (IY+d) (d = -3)
        assert_eq!(test_core.memory.dummy_memory[3], 0x8);


    }
}
