use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interuptor;
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
           interuptor: &mut interuptor::Interuptor) -> () where M: memory::MemoryRW{
        match op_code {
            // Extended op codes, not executed directly
            0xcb => { Self::execute_cb(clock, memory, pc_state, ports, interuptor);}
            0xdd => { Self::execute_dd(clock, memory, pc_state, ports, interuptor);}
            0xed => { Self::execute_ed(clock, memory, pc_state, ports, interuptor);}
            0xfd => { Self::execute_fd(clock, memory, pc_state, ports, interuptor);}

//            0xfb => { instruction_set::ei(clock, memory, clocks, pc_state, interupt, poll_interupts, step_func);}

            0x00 => { instruction_set::noop(clock, pc_state);}
            0x01 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.bc_reg);} // LD dd, nn : for BC
            0x02 => { instruction_set::ld_mem_r(clock, memory, pc_state.get_a(), &mut pc_state.pc_reg, &mut pc_state.bc_reg);} // LD (BC), A
            0x12 => { instruction_set::ld_mem_r(clock, memory, pc_state.get_a(), &mut pc_state.pc_reg, &mut pc_state.de_reg);} // LD (DE), A
//            0x07 => { instruction_set::rlca(clock, memory, pc_state);}  //RLCA

            n if (n & 0b11001111 == 0b00001001) => {
                let ss = (n >> 4) & 0x3;
                instruction_set::add16(clock, select_16_bit_read_register(pc_state, ss), 
                                       &mut pc_state.pc_reg, &mut pc_state.hl_reg, &mut pc_state.af_reg);
            }

//            0x0f => { instruction_set::rrca(clock, memory, pc_state);}
//            0x10 => { instruction_set::djnz(clock, memory, pc_state);} // DJNZ n
            0x11 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.de_reg);} // LD DE, nn
            0x21 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD HL, nn
            0x2a => { instruction_set::ld_r16_mem(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD HL, (nn)
            0x31 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg);} // LD DE, nn
            0x20 => { instruction_set::jrnz_e(clock, memory, pc_state);} // JR NZ, e
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
// 
            0x36 => { instruction_set::ld_mem_n(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD (HL), n
// 
// 
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
                    instruction_set::cp_r(clock, memory, 
                            select_8_bit_read_register(pc_state, reg_index), // gets the appropriate register getter fromt the supplied op-code
                            pc_state); // CP r
                }

            // JP cc, nn instructions
            // opcode: 0b11ccc010 
            n if (n & 0b11000111 == 0b11000010) => {
                    let condition = match (n >> 3) & 0b111 {
                        0b000 => {pc_state.get_f().get_z() == 0}  // Non-Zero (NZ)     Z
                        0b001 => {pc_state.get_f().get_z() == 1}  // Zero (Z)          Z
                        0b010 => {pc_state.get_f().get_c() == 0}  // No Carry (NC)     C
                        0b011 => {pc_state.get_f().get_c() == 1}  // Carry (C)         C
                        0b100 => {pc_state.get_f().get_pv() == 0} // Parity Odd (PO)   P/V
                        0b101 => {pc_state.get_f().get_pv() == 1} // Parity Even (PE)  P/V
                        0b110 => {pc_state.get_f().get_s() == 0}  // Sign Positive (P) S
                        0b111 => {pc_state.get_f().get_s() == 1}  // Sign Negative (M) S
                        _ => {panic!("Code path that was thought to be unreachable was reached! {}", n);}
                    };
                    instruction_set::jump_cc_nn(clock, memory, pc_state, condition);
                }
// 
//            0xd3 => { instruction_set::out_n_A(clock, memory, pc_state, self.ports);} // OUT (n), cpu_state->A
//            0xd9 => { instruction_set::exx(clock, memory, pc_state);} // EXX
// 
//            0xe6 => { instruction_set::and_n(clock, memory, pc_state);} // AND n
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
                    let dst_fn = get_8_bit_register_set_function ((op_code >> 3) & 0x7);

                    instruction_set::ld_r_r(clock, 
                            select_8_bit_read_register(pc_state, op_code & 0x7), // gets the appropriate register getter fromt the supplied op-code
                            pc_state, dst_fn);
                }
//
//            0xc9 => { instruction_set::ret(clock, memory, pc_state);} // RET
//
//            0x08 => { instruction_set::ex(clock, memory, pc_state);}
//            0x17 => { instruction_set::rla(clock, memory, pc_state);}
            0x18 => { instruction_set::jr_e(clock, memory, pc_state);}
//            0x1f => { instruction_set::rra(clock, memory, pc_state);}
            0x22 => { instruction_set::ld_mem_nn_hl(clock, memory, pc_state);}
//            0x27 => { instruction_set::daa(clock, memory, pc_state);}
//            0x2f => { instruction_set::cpl(clock, memory, pc_state);}
            0x30 => { instruction_set::jrnc_e(clock, memory, pc_state);}
            0x32 => { instruction_set::ld_nn_r(clock, memory, pc_state.get_a(), &mut pc_state.pc_reg);}
//            0x37 => { instruction_set::scf(clock, memory, pc_state);}
            0x38 => { instruction_set::jrc_e(clock, memory, pc_state);}
//            0x3f => { instruction_set::ccf(clock, memory, pc_state);}
//            0x76 => { instruction_set::halt(clock, memory, pc_state);}
//            0x86 => { instruction_set::add_hl(clock, memory, pc_state);}
//
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
                    let condition = match (n >> 3) & 0b111 {
                        0b000 => {pc_state.get_f().get_z() == 0}  // Non-Zero (NZ)     Z
                        0b001 => {pc_state.get_f().get_z() == 1}  // Zero (Z)          Z
                        0b010 => {pc_state.get_f().get_c() == 0}  // No Carry (NC)     C
                        0b011 => {pc_state.get_f().get_c() == 1}  // Carry (C)         C
                        0b100 => {pc_state.get_f().get_pv() == 0} // Parity Odd (PO)   P/V
                        0b101 => {pc_state.get_f().get_pv() == 1} // Parity Even (PE)  P/V
                        0b110 => {pc_state.get_f().get_s() == 0}  // Sign Positive (P) S
                        0b111 => {pc_state.get_f().get_s() == 1}  // Sign Negative (M) S
                        _ => {panic!("Code path that was thought to be unreachable was reached! {}", n);}
                    };
                    instruction_set::call_cc_nn(clock, memory, pc_state, condition);
                }
            0xcd => { instruction_set::call_nn(clock, memory, pc_state);}

//            0x8e => { instruction_set::adc_hl(clock, memory, pc_state);}
//            0x96 => { instruction_set::sub_hl(clock, memory, pc_state);}
//            0x9e => { instruction_set::sbc_a_hl(clock, memory, pc_state);}
//            0xa6 => { instruction_set::and_hl(clock, memory, pc_state);}
//            0xae => { instruction_set::xor_hl(clock, memory, pc_state);}
//            0xb6 => { instruction_set::or_hl(clock, memory, pc_state);}
            0xbe => { instruction_set::cp_hl(clock, memory, pc_state);}
//            0xc0 => { instruction_set::ret_nz(clock, memory, pc_state);}
//            0xc1 => { instruction_set::pop(clock, memory, pc_state, &mut pc_state.bc_reg);}
            0xc3 => { instruction_set::jp_nn(clock, memory, pc_state);}
//            0xc5 => { instruction_set::push(clock, memory, pc_state, &mut pc_state.bc_reg);}
//            0xc6 => { instruction_set::add_n(clock, memory, pc_state);}
//            0xc7 => { instruction_set::rst(clock, memory, pc_state, 0x00);} // RST_00
//            0xc8 => { instruction_set::rst_z(clock, memory, pc_state);}
//            0xce => { instruction_set::adc_nn(clock, memory, pc_state);}
//            0xcf => { instruction_set::rst(clock, memory, pc_state, 0x08);} // RST_08
//            0xd0 => { instruction_set::ret_nc(clock, memory, pc_state);}
//            0xd1 => { instruction_set::pop(clock, memory, pc_state, &mut pc_state.de_reg);}
//            0xd5 => { instruction_set::push(clock, memory, pc_state, &mut pc_state.de_reg);}
//            0xd6 => { instruction_set::sub_n(clock, memory, pc_state);}
//            0xd7 => { instruction_set::rst(clock, memory, pc_state, 0x10);} // RST_10
//            0xd8 => { instruction_set::ret_c(clock, memory, pc_state);}
            0xdb => { instruction_set::in_a_n(clock, memory, pc_state, ports);}
//            0xde => { instruction_set::sbc_n(clock, memory, pc_state);}
//            0xdf => { instruction_set::rst(clock, memory, pc_state, 0x18);} // RST_18
//            0xe0 => { instruction_set::ret_po(clock, memory, pc_state);}
//            0xe1 => { instruction_set::pop(clock, memory, pc_state, &mut pc_state.hl_reg);}
//            0xe3 => { instruction_set::ex_sp_hl(clock, memory, pc_state);}
//            0xe5 => { instruction_set::push(clock, memory, pc_state, &mut pc_state.hl_reg);}
//            0xe7 => { instruction_set::rst(clock, memory, pc_state, 0x20);} // RST_20
//            0xe8 => { instruction_set::ret_pe(clock, memory, pc_state);}
            0xe9 => { instruction_set::jp_hl(clock, &pc_state.hl_reg, &mut pc_state.pc_reg);}
//            0xeb => { instruction_set::ex_de_hl(clock, memory, pc_state);}
//            0xee => { instruction_set::xor_n(clock, memory, pc_state);}
//            0xef => { instruction_set::rst(clock, memory, pc_state, 0x28);} // RST_28
//            0xf0 => { instruction_set::ret_p(clock, memory, pc_state);}
//            0xf1 => { instruction_set::pop_af(clock, memory, pc_state);}
            0xf3 => { instruction_set::di(clock, pc_state);}
//            0xf5 => { instruction_set::push_af(clock, memory, pc_state);}
//            0xf6 => { instruction_set::or_n(clock, memory, pc_state);}
//            0xf7 => { instruction_set::rst(clock, memory, pc_state, 0x30);} // RST_30
//            0xf8 => { instruction_set::ret_m(clock, memory, pc_state);}
            0xf9 => { instruction_set::ld_sp_hl(clock, &pc_state.hl_reg, &mut pc_state.pc_reg, &mut pc_state.sp_reg);}
//            0xff => { instruction_set::rst(clock, memory, pc_state, 0x38);} // RST_38
        
            _ => {panic!("Opcode not implemented: {:x}", op_code); }

        }
    } 

    // Extended instructions
    pub fn execute_cb<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () where M: memory::MemoryRW {
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
//
//        # Non-masked op codes
//        self.instruction_cb_lookup[0x00] = instructions.RLC_r(clock, memory, pc_state, self._reg_wrapper_b); # RLC r, cpu_state->B
//        self.instruction_cb_lookup[0x01] = instructions.RLC_r(clock, memory, pc_state, self._reg_wrapper_c); # RLC r, cpu_state->C
//        self.instruction_cb_lookup[0x02] = instructions.RLC_r(clock, memory, pc_state, self._reg_wrapper_d); # RLC r, cpu_state->D
//        self.instruction_cb_lookup[0x03] = instructions.RLC_r(clock, memory, pc_state, self._reg_wrapper_e); # RLC r, cpu_state->E
//        self.instruction_cb_lookup[0x04] = instructions.RLC_r(clock, memory, pc_state, self._reg_wrapper_h); # RLC r, cpu_state->H
//        self.instruction_cb_lookup[0x05] = instructions.RLC_r(clock, memory, pc_state, self._reg_wrapper_l); # RLC r, cpu_state->L
//        self.instruction_cb_lookup[0x07] = instructions.RLC_r(clock, memory, pc_state, self._reg_wrapper_a); # RLC r, cpu_state->A
//        self.instruction_cb_lookup[0x06] = instructions.RLC_HL(clock, memory, pc_state); # RLC b, cpu_state->HL
//
//        self.instruction_cb_lookup[0x08] = instructions.RRC_r(clock, memory, pc_state, self._reg_wrapper_b); # RRC r, cpu_state->B
//        self.instruction_cb_lookup[0x09] = instructions.RRC_r(clock, memory, pc_state, self._reg_wrapper_c); # RRC r, cpu_state->C
//        self.instruction_cb_lookup[0x0A] = instructions.RRC_r(clock, memory, pc_state, self._reg_wrapper_d); # RRC r, cpu_state->D
//        self.instruction_cb_lookup[0x0B] = instructions.RRC_r(clock, memory, pc_state, self._reg_wrapper_e); # RRC r, cpu_state->E
//        self.instruction_cb_lookup[0x0C] = instructions.RRC_r(clock, memory, pc_state, self._reg_wrapper_h); # RRC r, cpu_state->H
//        self.instruction_cb_lookup[0x0D] = instructions.RRC_r(clock, memory, pc_state, self._reg_wrapper_l); # RRC r, cpu_state->L
//        self.instruction_cb_lookup[0x0F] = instructions.RRC_r(clock, memory, pc_state, self._reg_wrapper_a); # RRC r, cpu_state->A
//        self.instruction_cb_lookup[0x0E] = instructions.RRC_HL(clock, memory, pc_state); # RRC b, cpu_state->HL
//
//        self.instruction_cb_lookup[0x10] = instructions.RL_r(clock, memory, pc_state, self._reg_wrapper_b); # RL r, cpu_state->B
//        self.instruction_cb_lookup[0x11] = instructions.RL_r(clock, memory, pc_state, self._reg_wrapper_c); # RL r, cpu_state->C
//        self.instruction_cb_lookup[0x12] = instructions.RL_r(clock, memory, pc_state, self._reg_wrapper_d); # RL r, cpu_state->D
//        self.instruction_cb_lookup[0x13] = instructions.RL_r(clock, memory, pc_state, self._reg_wrapper_e); # RL r, cpu_state->E
//        self.instruction_cb_lookup[0x14] = instructions.RL_r(clock, memory, pc_state, self._reg_wrapper_h); # RL r, cpu_state->H
//        self.instruction_cb_lookup[0x15] = instructions.RL_r(clock, memory, pc_state, self._reg_wrapper_l); # RL r, cpu_state->L
//        self.instruction_cb_lookup[0x17] = instructions.RL_r(clock, memory, pc_state, self._reg_wrapper_a); # RL r, cpu_state->A
//
//        self.instruction_cb_lookup[0x18] = instructions.RR_r(clock, memory, pc_state, self._reg_wrapper_b); # RR r, cpu_state->B
//        self.instruction_cb_lookup[0x19] = instructions.RR_r(clock, memory, pc_state, self._reg_wrapper_c); # RR r, cpu_state->C
//        self.instruction_cb_lookup[0x1A] = instructions.RR_r(clock, memory, pc_state, self._reg_wrapper_d); # RR r, cpu_state->D
//        self.instruction_cb_lookup[0x1B] = instructions.RR_r(clock, memory, pc_state, self._reg_wrapper_e); # RR r, cpu_state->E
//        self.instruction_cb_lookup[0x1C] = instructions.RR_r(clock, memory, pc_state, self._reg_wrapper_h); # RR r, cpu_state->H
//        self.instruction_cb_lookup[0x1D] = instructions.RR_r(clock, memory, pc_state, self._reg_wrapper_l); # RR r, cpu_state->L
//        self.instruction_cb_lookup[0x1F] = instructions.RR_r(clock, memory, pc_state, self._reg_wrapper_a); # RR r, cpu_state->A
//
//        self.instruction_cb_lookup[0x20] = instructions.SLA_r(clock, memory, pc_state, self._reg_wrapper_b); # SLA r, cpu_state->B
//        self.instruction_cb_lookup[0x21] = instructions.SLA_r(clock, memory, pc_state, self._reg_wrapper_c); # SLA r, cpu_state->C
//        self.instruction_cb_lookup[0x22] = instructions.SLA_r(clock, memory, pc_state, self._reg_wrapper_d); # SLA r, cpu_state->D
//        self.instruction_cb_lookup[0x23] = instructions.SLA_r(clock, memory, pc_state, self._reg_wrapper_e); # SLA r, cpu_state->E
//        self.instruction_cb_lookup[0x24] = instructions.SLA_r(clock, memory, pc_state, self._reg_wrapper_h); # SLA r, cpu_state->H
//        self.instruction_cb_lookup[0x25] = instructions.SLA_r(clock, memory, pc_state, self._reg_wrapper_l); # SLA r, cpu_state->L
//        self.instruction_cb_lookup[0x27] = instructions.SLA_r(clock, memory, pc_state, self._reg_wrapper_a); # SLA r, cpu_state->A
//        self.instruction_cb_lookup[0x26] = instructions.SLA_HL(clock, memory, pc_state); # SLA b, cpu_state->HL
//
//        self.instruction_cb_lookup[0x28] = instructions.SRA_r(clock, memory, pc_state, self._reg_wrapper_b); # SRA r, cpu_state->B
//        self.instruction_cb_lookup[0x29] = instructions.SRA_r(clock, memory, pc_state, self._reg_wrapper_c); # SRA r, cpu_state->C
//        self.instruction_cb_lookup[0x2A] = instructions.SRA_r(clock, memory, pc_state, self._reg_wrapper_d); # SRA r, cpu_state->D
//        self.instruction_cb_lookup[0x2B] = instructions.SRA_r(clock, memory, pc_state, self._reg_wrapper_e); # SRA r, cpu_state->E
//        self.instruction_cb_lookup[0x2C] = instructions.SRA_r(clock, memory, pc_state, self._reg_wrapper_h); # SRA r, cpu_state->H
//        self.instruction_cb_lookup[0x2D] = instructions.SRA_r(clock, memory, pc_state, self._reg_wrapper_l); # SRA r, cpu_state->L
//        self.instruction_cb_lookup[0x2F] = instructions.SRA_r(clock, memory, pc_state, self._reg_wrapper_a); # SRA r, cpu_state->A
//        self.instruction_cb_lookup[0x2E] = instructions.SRA_HL(clock, memory, pc_state); # SRA b, cpu_state->HL
//
//        self.instruction_cb_lookup[0x30] = instructions.SLL_r(clock, memory, pc_state, self._reg_wrapper_b); # SLL r, cpu_state->B
//        self.instruction_cb_lookup[0x31] = instructions.SLL_r(clock, memory, pc_state, self._reg_wrapper_c); # SLL r, cpu_state->C
//        self.instruction_cb_lookup[0x32] = instructions.SLL_r(clock, memory, pc_state, self._reg_wrapper_d); # SLL r, cpu_state->D
//        self.instruction_cb_lookup[0x33] = instructions.SLL_r(clock, memory, pc_state, self._reg_wrapper_e); # SLL r, cpu_state->E
//        self.instruction_cb_lookup[0x34] = instructions.SLL_r(clock, memory, pc_state, self._reg_wrapper_h); # SLL r, cpu_state->H
//        self.instruction_cb_lookup[0x35] = instructions.SLL_r(clock, memory, pc_state, self._reg_wrapper_l); # SLL r, cpu_state->L
//        self.instruction_cb_lookup[0x37] = instructions.SLL_r(clock, memory, pc_state, self._reg_wrapper_a); # SLL r, cpu_state->A
//        self.instruction_cb_lookup[0x36] = instructions.SLL_HL(clock, memory, pc_state); # SLL b, cpu_state->HL
//
//        self.instruction_cb_lookup[0x38] = instructions.SRL_r(clock, memory, pc_state, self._reg_wrapper_b); # SRL r, cpu_state->B
//        self.instruction_cb_lookup[0x39] = instructions.SRL_r(clock, memory, pc_state, self._reg_wrapper_c); # SRL r, cpu_state->C
//        self.instruction_cb_lookup[0x3A] = instructions.SRL_r(clock, memory, pc_state, self._reg_wrapper_d); # SRL r, cpu_state->D
//        self.instruction_cb_lookup[0x3B] = instructions.SRL_r(clock, memory, pc_state, self._reg_wrapper_e); # SRL r, cpu_state->E
//        self.instruction_cb_lookup[0x3C] = instructions.SRL_r(clock, memory, pc_state, self._reg_wrapper_h); # SRL r, cpu_state->H
//        self.instruction_cb_lookup[0x3D] = instructions.SRL_r(clock, memory, pc_state, self._reg_wrapper_l); # SRL r, cpu_state->L
//        self.instruction_cb_lookup[0x3F] = instructions.SRL_r(clock, memory, pc_state, self._reg_wrapper_a); # SRL r, cpu_state->A
//        self.instruction_cb_lookup[0x3E] = instructions.SRL_HL(clock, memory, pc_state); # SRA b, cpu_state->HL
            _ => {panic!("Extended(0xCB) Opcode not implemented: {:x}", op_code); }

        }
    } 

    // Extended instructions
    pub fn execute_dd<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () where M: memory::MemoryRW {
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
            0xBE => { extended_instruction_set::cp_i_d(clock, memory, pc_state.ix_reg.get(), pc_state);}

            n if (n & 0b11001111 == 0b00001001) => {
                let ss = (n >> 4) & 0x3;
                extended_instruction_set::add16(clock, select_16_bit_read_register(pc_state, ss), 
                                                &mut pc_state.pc_reg, &mut pc_state.ix_reg, &mut pc_state.af_reg);
            }

            _ => {panic!("Extended(0xDD) Opcode not implemented: {:x}", op_code); }

//            0x23 => { extended_instruction_set::INC_16(clock, memory, pc_state, self._reg_wrapper_ix, 10,2);}
//            0x2B => { extended_instruction_set::DEC_16(clock, memory, pc_state, self._reg_wrapper_ix, 10,2);}
//
//            0x34 => { extended_instruction_set::INC_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0x35 => { extended_instruction_set::DEC_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}

//
//            0x86 => { extended_instruction_set::ADDA_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0x8E => { extended_instruction_set::ADC_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0x96 => { extended_instruction_set::SUB_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0xA6 => { extended_instruction_set::AND_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0xAE => { extended_instruction_set::XOR_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0xB6 => { extended_instruction_set::OR_I_d(clock, memory, pc_state, self._reg_wrapper_ix);}
//
//            0xE1 => { extended_instruction_set::POP_I(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0xE3 => { extended_instruction_set::EX_SP_I(clock, memory, pc_state, self._reg_wrapper_ix);}
//            0xE5 => { extended_instruction_set::PUSH_I(clock, memory, pc_state, self._reg_wrapper_ix);}
        }
    } 
    // Extended instructions
    pub fn execute_fd<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () where M: memory::MemoryRW {
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
            0xBE => { extended_instruction_set::cp_i_d(clock, memory, pc_state.ix_reg.get(), pc_state);}
            n if (n & 0b11001111 == 0b00001001) => {
                let ss = (n >> 4) & 0x3;
                extended_instruction_set::add16(clock, select_16_bit_read_register(pc_state, ss), 
                                                &mut pc_state.pc_reg, &mut pc_state.iy_reg, &mut pc_state.af_reg);
            }

             _ => {panic!("Extended(0xFD) Opcode not implemented: {:x}", op_code); }

//            0x23 => { extended_instruction_set::INC_16(clock, memory, pc_state, self._reg_wrapper_iy, 10,2);}}
//            0x2B => { extended_instruction_set::DEC_16(clock, memory, pc_state, self._reg_wrapper_iy, 10,2);}}
//
//            0x34 => { extended_instruction_set::INC_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0x35 => { extended_instruction_set::DEC_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//
//
//
//            0x86 => { extended_instruction_set::ADDA_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0x8E => { extended_instruction_set::ADC_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0x96 => { extended_instruction_set::SUB_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0xA6 => { extended_instruction_set::AND_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0xAE => { extended_instruction_set::XOR_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0xB6 => { extended_instruction_set::OR_I_d(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0xE1 => { extended_instruction_set::POP_I(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0xE3 => { extended_instruction_set::EX_SP_I(clock, memory, pc_state, self._reg_wrapper_iy);}
//            0xE5 => { extended_instruction_set::PUSH_I(clock, memory, pc_state, self._reg_wrapper_iy);}
        }
    } 
    // Extended instructions
    pub fn execute_ed<M>(clock: &mut clocks::Clock, 
           memory: &mut M, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () where M: memory::MemoryRW {
        let op_code = memory.read(pc_state.get_pc() + 1);
        println!("clock: {}, op_code: {:x}, pc: {}", clock.cycles, op_code, pc_state.get_pc());

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            0x56 => { instruction_set::im_1(clock, pc_state);} 

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

            _ => {panic!("Extended(0xED) Opcode not implemented: {:x}", op_code); }

//            0x40 => { extended_instruction_set::IN_r_C(clock, memory, pc_state, ports, self._reg_wrapper_b);}
//            0x48 => { extended_instruction_set::IN_r_C(clock, memory, pc_state, ports, self._reg_wrapper_c);}
//            0x50 => { extended_instruction_set::IN_r_C(clock, memory, pc_state, ports, self._reg_wrapper_d);}
//            0x58 => { extended_instruction_set::IN_r_C(clock, memory, pc_state, ports, self._reg_wrapper_e);}
//            0x60 => { extended_instruction_set::IN_r_C(clock, memory, pc_state, ports, self._reg_wrapper_h);}
//            0x68 => { extended_instruction_set::IN_r_C(clock, memory, pc_state, ports, self._reg_wrapper_l);}
//            0x78 => { extended_instruction_set::IN_r_C(clock, memory, pc_state, ports, self._reg_wrapper_a);}
//
//            0x41 => { extended_instruction_set::OUT_C_r(clock, memory, pc_state, ports, self._reg_wrapper_b);}
//            0x49 => { extended_instruction_set::OUT_C_r(clock, memory, pc_state, ports, self._reg_wrapper_c);}
//            0x51 => { extended_instruction_set::OUT_C_r(clock, memory, pc_state, ports, self._reg_wrapper_d);}
//            0x59 => { extended_instruction_set::OUT_C_r(clock, memory, pc_state, ports, self._reg_wrapper_e);}
//            0x61 => { extended_instruction_set::OUT_C_r(clock, memory, pc_state, ports, self._reg_wrapper_h);}
//            0x69 => { extended_instruction_set::OUT_C_r(clock, memory, pc_state, ports, self._reg_wrapper_l);}
//            0x79 => { extended_instruction_set::OUT_C_r(clock, memory, pc_state, ports, self._reg_wrapper_a);}
//
//            0x42 => { extended_instruction_set::SBC_HL_r16(clock, memory, pc_state, self._reg_wrapper_bc);}
//            0x44 => { extended_instruction_set::NEG(clock, memory, pc_state);}
//            0x4A => { extended_instruction_set::ADC_HL_r16(clock, memory, pc_state, self._reg_wrapper_bc);}
//            0x4D => { extended_instruction_set::RETI(clock, memory, pc_state);}
//            0x52 => { extended_instruction_set::SBC_HL_r16(clock, memory, pc_state, self._reg_wrapper_de);}
//            0x56 => { extended_instruction_set::IM_1(clock, memory, pc_state);}
//            0x5A => { extended_instruction_set::ADC_HL_r16(clock, memory, pc_state, self._reg_wrapper_de);}
//            0x62 => { extended_instruction_set::SBC_HL_r16(clock, memory, pc_state, self._reg_wrapper_hl);}
//            0x67 => { extended_instruction_set::RRD(clock, memory, pc_state);}
//            0x6A => { extended_instruction_set::ADC_HL_r16(clock, memory, pc_state, self._reg_wrapper_hl);}
//            0x72 => { extended_instruction_set::SBC_HL_r16(clock, memory, pc_state, self._reg_wrapper_sp);}
//            0x7A => { extended_instruction_set::ADC_HL_r16(clock, memory, pc_state, self._reg_wrapper_sp);}
//            0xA0 => { extended_instruction_set::LDI(clock, memory, pc_state);}
//            0xA1 => { extended_instruction_set::CPI(clock, memory, pc_state);}
//            0xA2 => { extended_instruction_set::INI(clock, memory, pc_state, ports);}
//            0xA3 => { extended_instruction_set::OUTI(clock, memory, pc_state, ports);}
//            0xAB => { extended_instruction_set::OUTD(clock, memory, pc_state, ports);}
//            0xB0 => { extended_instruction_set::LDIR(clock, memory, pc_state);}
//            0xB1 => { extended_instruction_set::CPIR(clock, memory, pc_state);}
//            0xB3 => { extended_instruction_set::OTIR(clock, memory, pc_state, ports);}
//            0xB8 => { extended_instruction_set::LDDR(clock, memory, pc_state);}
        }
    } 
}

#[cfg(test)]
mod tests {
    use crate::sega::cpu::instructions;
    use crate::sega::cpu::pc_state;
    use crate::sega::memory::memory;
    use crate::sega::clocks;
    use crate::sega::interuptor;
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
        pub interuptor: interuptor::Interuptor,
    }
    impl TestCore {
        pub fn new() -> Self {
            Self {
                clock: clocks::Clock::new(),
                memory: TestMemory::new(),
                pc_state: pc_state::PcState::new(),
                ports: ports::Ports::new(),
                interuptor: interuptor::Interuptor::new(),
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
        instructions::Instruction::execute(0b01000001, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interuptor); // LD r,'r  C -> B
        assert_eq!(test_core.pc_state.get_b(), 0x42);
        assert_eq!(test_core.clock.cycles, 4);
    }

    #[test]
    fn test_jump_functions() {
        let mut test_core = TestCore::new();

        test_core.pc_state.set_hl(0x4233);
        test_core.pc_state.set_pc(0x2003);
        instructions::Instruction::execute(0xE9, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interuptor); // JP (HL)
        assert_eq!(test_core.pc_state.get_pc(), 0x4233);
    }

    #[test]
    fn test_dec_functions() {
        let mut test_core = TestCore::new();

        test_core.pc_state.set_h(0x80);
        let mut flags = test_core.pc_state.get_f();
        flags.set_c(1);
        test_core.pc_state.set_f(flags);
        instructions::Instruction::execute(0b00100101, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interuptor); // dec_r, for h
        assert_eq!(test_core.pc_state.get_h(), 0x7F);
        assert_eq!(test_core.pc_state.get_f().get_h(), 1);
        assert_eq!(test_core.pc_state.get_f().get_c(), 1);
        assert_eq!(test_core.pc_state.get_f().get_s(), 0);
    }

    #[test]
    fn test_specific_opcodes() {
        let mut test_core = TestCore::new();

        test_core.memory.dummy_memory = vec![0x00];
        instructions::Instruction::execute(0x00, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interuptor); // no-op
        assert_eq!(test_core.pc_state.get_pc(), 0x1);
        assert_eq!(test_core.clock.cycles, 4);

        // LD dd, nn: for BC
        // Reset the PC counter.
        test_core.pc_state.set_pc(0);
        test_core.clock.cycles = 0;
        let test_op_code = 0x01;
        test_core.memory.dummy_memory = vec![test_op_code, 0x10, 0x33]; // Op-code to test
        instructions::Instruction::execute(test_op_code, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interuptor);
        assert_eq!(test_core.pc_state.get_pc(), 0x3);
        assert_eq!(test_core.pc_state.get_bc(), 0x3310);
        assert_eq!(test_core.clock.cycles, 10);
    }

    #[test]
    fn test_opcode_cycle_times() {
        let mut test_core = TestCore::new();

        fn test_op_code_cycle_count(test_core: &mut TestCore, op_code: Vec<u8>, expected_pc: u16, expected_cycles: u32) {
            // Reset the PC counter.
            test_core.pc_state.set_pc(0);
            test_core.clock.cycles = 0;
            let initial_op_code = op_code[0];
            test_core.memory.dummy_memory = op_code; // Op-code to test
            instructions::Instruction::execute(initial_op_code, &mut test_core.clock, &mut test_core.memory, &mut test_core.pc_state, &mut test_core.ports, &mut test_core.interuptor);
            assert_eq!(test_core.pc_state.get_pc(), expected_pc);
            assert_eq!(test_core.clock.cycles, expected_cycles);
        }

        test_op_code_cycle_count(&mut test_core, vec![0x00], 1, 4); // no-op
        test_op_code_cycle_count(&mut test_core, vec![0x01, 0x10, 0x33], 3, 10); // LD dd, nn

    }
}
