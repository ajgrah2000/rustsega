use super::pc_state;
use super::super::memory::memory;
use super::super::clocks;
use super::super::interuptor;
use super::super::ports;
use super::instruction_set;

pub struct Instruction {
}

fn select_8_bit_read_register (reg_select: u8) -> impl Fn(&pc_state::PcState) -> u8 {
    let src = move |state: &pc_state::PcState| match reg_select & 0x7 {
        0 => {state.get_b()}
        1 => {state.get_c()}
        2 => {state.get_d()}
        3 => {state.get_e()}
        4 => {state.get_h()}
        5 => {state.get_l()}
        7 => {state.get_a()}
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

impl Instruction {
    pub fn execute(op_code: u8, clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
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
//            0x03 => { instruction_set::inc_bc(clock, memory, pc_state);} // INC cpu_state->BC
//            0x04 => { instruction_set::inc_r(clock, memory, pc_state, self._reg_wrapper_b);} // INC cpu_state->B
//            0x07 => { instruction_set::rlca(clock, memory, pc_state);}  //RLCA
//            0x09 => { instruction_set::add16(clock, memory, pc_state, &mut pc_state.hl_reg, &mut pc_state.bc_reg,11);}
//            0x0b => { instruction_set::dec_16(clock, memory, pc_state, &mut pc_state.bc_reg, 6);}
//            0x0c => { instruction_set::inc_r(clock, memory, pc_state, self._reg_wrapper_c);} // INC C
//            0x0f => { instruction_set::rrca(clock, memory, pc_state);}
//            0x10 => { instruction_set::djnz(clock, memory, pc_state);} // DJNZ n
            0x11 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.de_reg);} // LD DE, nn
            0x21 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD HL, nn
            0x2a => { instruction_set::ld_r16_mem(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD HL, (nn)
            0x31 => { instruction_set::ld_16_nn(clock, memory, &mut pc_state.pc_reg, &mut pc_state.sp_reg);} // LD DE, nn
//            0x13 => { instruction_set::inc_16(clock, memory, pc_state, &mut pc_state.de_reg, 6);}
//            0x14 => { instruction_set::inc_r(clock, memory, pc_state, self._reg_wrapper_d);} // INC D
//            0x19 => { instruction_set::add16(clock, memory, pc_state, &mut pc_state.hl_reg, &mut pc_state.de_reg,11);}
//            0x1b => { instruction_set::dec_16(clock, memory, pc_state, &mut pc_state.de_reg, 6);}
// 
//            0x1c => { instruction_set::inc_r(clock, memory, pc_state, self._reg_wrapper_e);} // INC E
// 
            0x20 => { instruction_set::jrnz_e(clock, memory, pc_state);} // JR NZ, e
// 
//            0x23 => { instruction_set::inc_16(clock, memory, pc_state, &mut pc_state.hl_reg, 6);}
// 
//            0x24 => { instruction_set::inc_r(clock, memory, pc_state, self._reg_wrapper_h);} // INC H
            0x28 => { instruction_set::jrz_e(clock, memory, pc_state);} // JR Z, e
//            0x29 => { instruction_set::add16(clock, memory, pc_state, &mut pc_state.hl_reg, &mut pc_state.hl_reg,11);}
//            0x2b => { instruction_set::dec_16(clock, memory, pc_state, &mut pc_state.hl_reg, 6);}
// 
//            0x2c => { instruction_set::inc_r(clock, memory, pc_state, self._reg_wrapper_l);} // INC L
//            0x33 => { instruction_set::inc_16(clock, memory, pc_state, &mut pc_state.sp_reg, 6);}
//            0x34 => { instruction_set::inc_hl(clock, memory, pc_state);} // INC HL
//            0x35 => { instruction_set::dec_hl(clock, memory, pc_state);} // DEC HL
// 
            0x36 => { instruction_set::ld_mem_n(clock, memory, &mut pc_state.pc_reg, &mut pc_state.hl_reg);} // LD (HL), n
// 
//            0x39 => { instruction_set::add16(clock, memory, pc_state, &mut pc_state.hl_reg, &mut pc_state.sp_reg,11);}
// 
            0x3a => { instruction_set::ld_r8_mem(clock, memory, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)});} // LD A, (n)
//            0x3b => { instruction_set::dec_16(clock, memory, pc_state, &mut pc_state.sp_reg, 6);}
//            0x3c => { instruction_set::inc_r(clock, memory, pc_state, self._reg_wrapper_a);} // INC A

            // ld_mem_r instructions
            // opcode: 0b01110rrr 
            n if (n & 0b11111000 == 0b01110000) && (n != 0x76) => {
                    let get_reg_value_fn = select_8_bit_read_register(op_code & 0x7); // gets the appropriate register getter fromt the supplied op-code
                    instruction_set::ld_mem_r(clock, memory, get_reg_value_fn(pc_state),
                            &mut pc_state.pc_reg, &mut pc_state.hl_reg); // LD (HL), r
                }
// 
//            0x80 => { instruction_set::add_r(clock, memory, pc_state, self._reg_wrapper_b);} // ADD r, cpu_state->B
//            0x81 => { instruction_set::add_r(clock, memory, pc_state, self._reg_wrapper_c);} // ADD r, cpu_state->C
//            0x82 => { instruction_set::add_r(clock, memory, pc_state, self._reg_wrapper_d);} // ADD r, cpu_state->D
//            0x83 => { instruction_set::add_r(clock, memory, pc_state, self._reg_wrapper_e);} // ADD r, cpu_state->E
//            0x84 => { instruction_set::add_r(clock, memory, pc_state, self._reg_wrapper_h);} // ADD r, cpu_state->H
//            0x85 => { instruction_set::add_r(clock, memory, pc_state, self._reg_wrapper_l);} // ADD r, cpu_state->L
//            0x87 => { instruction_set::add_r(clock, memory, pc_state, self._reg_wrapper_a);} // ADD r, cpu_state->A
//
//            0x90 => { instruction_set::sub_r(clock, memory, pc_state, self._reg_wrapper_b);} // SUB r, cpu_state->B
//            0x91 => { instruction_set::sub_r(clock, memory, pc_state, self._reg_wrapper_c);} // SUB r, cpu_state->C
//            0x92 => { instruction_set::sub_r(clock, memory, pc_state, self._reg_wrapper_d);} // SUB r, cpu_state->D
//            0x93 => { instruction_set::sub_r(clock, memory, pc_state, self._reg_wrapper_e);} // SUB r, cpu_state->E
//            0x94 => { instruction_set::sub_r(clock, memory, pc_state, self._reg_wrapper_h);} // SUB r, cpu_state->H
//            0x95 => { instruction_set::sub_r(clock, memory, pc_state, self._reg_wrapper_l);} // SUB r, cpu_state->L
//            0x97 => { instruction_set::sub_a(clock, memory, pc_state);} // SUB r, cpu_state->A
// 
//            0xa0 => { instruction_set::and_r(clock, memory, pc_state, self._reg_wrapper_b);} // AND r, cpu_state->A
//            0xa1 => { instruction_set::and_r(clock, memory, pc_state, self._reg_wrapper_c);} // AND r, cpu_state->A
//            0xa2 => { instruction_set::and_r(clock, memory, pc_state, self._reg_wrapper_d);} // AND r, cpu_state->A
//            0xa3 => { instruction_set::and_r(clock, memory, pc_state, self._reg_wrapper_e);} // AND r, cpu_state->A
//            0xa4 => { instruction_set::and_r(clock, memory, pc_state, self._reg_wrapper_h);} // AND r, cpu_state->A
//            0xa5 => { instruction_set::and_r(clock, memory, pc_state, self._reg_wrapper_l);} // AND r, cpu_state->A
//            0xa7 => { instruction_set::and_a(clock, memory, pc_state);} // AND r, cpu_state->A
// 
//            0xa8 => { instruction_set::xor_r(clock, memory, pc_state, self._reg_wrapper_b);} // XOR r, cpu_state->A
//            0xa9 => { instruction_set::xor_r(clock, memory, pc_state, self._reg_wrapper_c);} // XOR r, cpu_state->A
//            0xaa => { instruction_set::xor_r(clock, memory, pc_state, self._reg_wrapper_d);} // XOR r, cpu_state->A
//            0xab => { instruction_set::xor_r(clock, memory, pc_state, self._reg_wrapper_e);} // XOR r, cpu_state->A
//            0xac => { instruction_set::xor_r(clock, memory, pc_state, self._reg_wrapper_h);} // XOR r, cpu_state->A
//            0xad => { instruction_set::xor_r(clock, memory, pc_state, self._reg_wrapper_l);} // XOR r, cpu_state->A
//            0xaf => { instruction_set::xor_a(clock, memory, pc_state);} // XOR r, cpu_state->A
// 
//            0xb0 => { instruction_set::or_r(clock, memory, pc_state, self._reg_wrapper_b);} // OR r, cpu_state->A
//            0xb1 => { instruction_set::or_r(clock, memory, pc_state, self._reg_wrapper_c);} // OR r, cpu_state->A
//            0xb2 => { instruction_set::or_r(clock, memory, pc_state, self._reg_wrapper_d);} // OR r, cpu_state->A
//            0xb3 => { instruction_set::or_e(clock, memory, pc_state);} // OR r, cpu_state->A
//            0xb4 => { instruction_set::or_r(clock, memory, pc_state, self._reg_wrapper_h);} // OR r, cpu_state->A
//            0xb5 => { instruction_set::or_r(clock, memory, pc_state, self._reg_wrapper_l);} // OR r, cpu_state->A
//            0xb7 => { instruction_set::or_a(clock, memory, pc_state);} // OR r, cpu_state->A
// 
            // cp_r instructions
            // opcode: 0b10111rrr 
            n if (n & 0b11111000 == 0b10111000) && (n != 0b11111110) => {
                    let get_reg_value_fn = select_8_bit_read_register(op_code & 0x7); // gets the appropriate register getter fromt the supplied op-code
                    instruction_set::cp_r(clock, memory, get_reg_value_fn(pc_state), pc_state); // CP r
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
//
//            0x05 => { instruction_set::dec_r(clock, memory, pc_state, self._reg_wrapper_b);} // DEC B
//            0x0d => { instruction_set::dec_r(clock, memory, pc_state, self._reg_wrapper_c);} // DEC C
//            0x15 => { instruction_set::dec_r(clock, memory, pc_state, self._reg_wrapper_d);} // DEC D
//            0x1d => { instruction_set::dec_r(clock, memory, pc_state, self._reg_wrapper_e);} // DEC E
//            0x25 => { instruction_set::dec_r(clock, memory, pc_state, self._reg_wrapper_h);} // DEC H
//            0x2d => { instruction_set::dec_r(clock, memory, pc_state, self._reg_wrapper_l);} // DEC L
//            0x3d => { instruction_set::dec_r(clock, memory, pc_state, self._reg_wrapper_a);} // DEC A
//

            // ld_r_mem instructions (eg // LD r, (HL)
            // op_code: 0b01rrr110  // LD r, (HL)
            n if (n & 0b11000111 == 0b01000110) && (((n >> 3) & 0x7) != 0x6) => {
                    let dst_fn = get_8_bit_register_set_function ((op_code >> 3) & 0x7);
                    instruction_set::ld_r_mem(clock, memory, pc_state, dst_fn, pc_state.hl_reg.get()); // LD r, (HL)
                }
            0x0a => { instruction_set::ld_r_mem(clock, memory, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.bc_reg.get());} // LD A, (BC)
            0x1a => { instruction_set::ld_r_mem(clock, memory, pc_state, |state: &mut pc_state::PcState, x| {state.set_a(x)}, pc_state.de_reg.get());} // LD A, (DE)

            // LD r,n
            // opcode: 0b00rrr110 nnnnnnnn
            n if (n & 0b11000111 == 0b00000110) && (((n >> 3) & 0x7) != 0x6) => {
                    let dst_fn = get_8_bit_register_set_function ((op_code >> 3) & 0x7);
                    instruction_set::ld_r(clock, memory, pc_state, dst_fn); // LD r, n
                }

            // ld_r_r instructions ( 0b01dddsss) 
            n if ((n & 0b11000000) == 0b01000000) && ((n & 0x07) != 0x6) && ((n & 0x38) != 0x3) => {
                    let get_reg_value_fn = select_8_bit_read_register(op_code & 0x7); // gets the appropriate register getter fromt the supplied op-code
                    let get_reg_set_fn = select_8_bit_read_register(op_code & 0x7); // gets the appropriate register getter fromt the supplied op-code
                    let dst_fn = get_8_bit_register_set_function ((op_code >> 3) & 0x7);

                    instruction_set::ld_r_r(clock, get_reg_value_fn(pc_state), pc_state, dst_fn);
                }
//
//            0xc9 => { instruction_set::ret(clock, memory, pc_state);} // RET
//
//            0x08 => { instruction_set::ex(clock, memory, pc_state);}
//            0x17 => { instruction_set::rla(clock, memory, pc_state);}
            0x18 => { instruction_set::jr_e(clock, memory, pc_state);}
//            0x1f => { instruction_set::rra(clock, memory, pc_state);}
            0x22 => { instruction_set::ld_nn_hl(clock, memory, pc_state);}
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
//            0x88 => { instruction_set::adc_r(clock, memory, pc_state, self._reg_wrapper_b);}
//            0x89 => { instruction_set::adc_r(clock, memory, pc_state, self._reg_wrapper_c);}
//            0x8a => { instruction_set::adc_r(clock, memory, pc_state, self._reg_wrapper_d);}
//            0x8b => { instruction_set::adc_r(clock, memory, pc_state, self._reg_wrapper_e);}
//            0x8c => { instruction_set::adc_r(clock, memory, pc_state, self._reg_wrapper_h);}
//            0x8d => { instruction_set::adc_r(clock, memory, pc_state, self._reg_wrapper_l);}
//            0x8f => { instruction_set::adc_r(clock, memory, pc_state, self._reg_wrapper_a);}
//            0x8e => { instruction_set::adc_hl(clock, memory, pc_state);}
//
//            0x96 => { instruction_set::sub_hl(clock, memory, pc_state);}
//
//            0x98 => { instruction_set::sbc_a_r(clock, memory, pc_state, self._reg_wrapper_b);}
//            0x99 => { instruction_set::sbc_a_r(clock, memory, pc_state, self._reg_wrapper_c);}
//            0x9a => { instruction_set::sbc_a_r(clock, memory, pc_state, self._reg_wrapper_d);}
//            0x9b => { instruction_set::sbc_a_r(clock, memory, pc_state, self._reg_wrapper_e);}
//            0x9c => { instruction_set::sbc_a_r(clock, memory, pc_state, self._reg_wrapper_h);}
//            0x9d => { instruction_set::sbc_a_r(clock, memory, pc_state, self._reg_wrapper_l);}
//            0x9f => { instruction_set::sbc_a_r(clock, memory, pc_state, self._reg_wrapper_a);}
//            0x9e => { instruction_set::sbc_a_hl(clock, memory, pc_state);}
//
//            0xa6 => { instruction_set::and_hl(clock, memory, pc_state);}
//            0xae => { instruction_set::xor_hl(clock, memory, pc_state);}
//            0xb6 => { instruction_set::or_hl(clock, memory, pc_state);}
            0xbe => { instruction_set::cp_hl(clock, memory, pc_state);}
//            0xc0 => { instruction_set::ret_nz(clock, memory, pc_state);}
//            0xc1 => { instruction_set::pop(clock, memory, pc_state, &mut pc_state.bc_reg);}
            0xc3 => { instruction_set::jp_nn(clock, memory, pc_state);}
//            0xc4 => { instruction_set::call_nz_nn(clock, memory, pc_state);}
//            0xc5 => { instruction_set::push(clock, memory, pc_state, &mut pc_state.bc_reg);}
//            0xc6 => { instruction_set::add_n(clock, memory, pc_state);}
//            0xc7 => { instruction_set::rst(clock, memory, pc_state, 0x00);} // RST_00
//            0xc8 => { instruction_set::rst_z(clock, memory, pc_state);}
//            0xcc => { instruction_set::call_z_nn(clock, memory, pc_state);}
//            0xcd => { instruction_set::call_nn(clock, memory, pc_state);}
//            0xce => { instruction_set::adc_nn(clock, memory, pc_state);}
//            0xcf => { instruction_set::rst(clock, memory, pc_state, 0x08);} // RST_08
//            0xd0 => { instruction_set::ret_nc(clock, memory, pc_state);}
//            0xd1 => { instruction_set::pop(clock, memory, pc_state, &mut pc_state.de_reg);}
//            0xd4 => { instruction_set::call_nc_nn(clock, memory, pc_state);}
//            0xd5 => { instruction_set::push(clock, memory, pc_state, &mut pc_state.de_reg);}
//            0xd6 => { instruction_set::sub_n(clock, memory, pc_state);}
//            0xd7 => { instruction_set::rst(clock, memory, pc_state, 0x10);} // RST_10
//            0xd8 => { instruction_set::ret_c(clock, memory, pc_state);}
            0xdb => { instruction_set::in_a_n(clock, memory, pc_state, ports);}
//            0xdc => { instruction_set::call_c_nn(clock, memory, pc_state);}
//            0xde => { instruction_set::sbc_n(clock, memory, pc_state);}
//            0xdf => { instruction_set::rst(clock, memory, pc_state, 0x18);} // RST_18
//            0xe0 => { instruction_set::ret_po(clock, memory, pc_state);}
//            0xe1 => { instruction_set::pop(clock, memory, pc_state, &mut pc_state.hl_reg);}
//            0xe3 => { instruction_set::ex_sp_hl(clock, memory, pc_state);}
//            0xe4 => { instruction_set::call_po_nn(clock, memory, pc_state);}
//            0xe5 => { instruction_set::push(clock, memory, pc_state, &mut pc_state.hl_reg);}
//            0xe7 => { instruction_set::rst(clock, memory, pc_state, 0x20);} // RST_20
//            0xe8 => { instruction_set::ret_pe(clock, memory, pc_state);}
            0xe9 => { instruction_set::jp_hl(clock, &pc_state.hl_reg, &mut pc_state.pc_reg);}
//            0xeb => { instruction_set::ex_de_hl(clock, memory, pc_state);}
//            0xec => { instruction_set::call_pe_nn(clock, memory, pc_state);}
//            0xee => { instruction_set::xor_n(clock, memory, pc_state);}
//            0xef => { instruction_set::rst(clock, memory, pc_state, 0x28);} // RST_28
//            0xf0 => { instruction_set::ret_p(clock, memory, pc_state);}
//            0xf1 => { instruction_set::pop_af(clock, memory, pc_state);}
            0xf3 => { instruction_set::di(clock, pc_state);}
//            0xf4 => { instruction_set::call_p_nn(clock, memory, pc_state);}
//            0xf5 => { instruction_set::push_af(clock, memory, pc_state);}
//            0xf6 => { instruction_set::or_n(clock, memory, pc_state);}
//            0xf7 => { instruction_set::rst(clock, memory, pc_state, 0x30);} // RST_30
//            0xf8 => { instruction_set::ret_m(clock, memory, pc_state);}
            0xf9 => { instruction_set::ld_sp_hl(clock, &pc_state.hl_reg, &mut pc_state.pc_reg, &mut pc_state.sp_reg);}
//            0xfc => { instruction_set::call_m_nn(clock, memory, pc_state);}
//            0xff => { instruction_set::rst(clock, memory, pc_state, 0x38);} // RST_38
        
            _ => {println!("Opcode not implemented: {:x}", op_code); }

        }
    } 

    // Extended instructions
    pub fn execute_cb(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            _ => {println!("Extended(0xCB) Opcode not implemented: {:x}", op_code); }

        }
    } 

    // Extended instructions
    pub fn execute_dd(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            _ => {println!("Extended(0xDD) Opcode not implemented: {:x}", op_code); }
        }
    } 
    // Extended instructions
    pub fn execute_fd(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            _ => {println!("Extended(0xFD) Opcode not implemented: {:x}", op_code); }
        }
    } 
    // Extended instructions
    pub fn execute_ed(clock: &mut clocks::Clock, 
           memory: &mut memory::MemoryAbsolute, 
           pc_state: &mut pc_state::PcState, 
           ports: &mut ports::Ports, 
           interuptor: &mut interuptor::Interuptor) -> () {
        let op_code = memory.read(pc_state.get_pc() + 1);
        println!("clock: {}, op_code: {:x}, pc: {}", clock.cycles, op_code, pc_state.get_pc());

        match op_code {
            0x00 => { instruction_set::noop(clock, pc_state);} 
            0x56 => { instruction_set::im_1(clock, pc_state);} 
            _ => {println!("Extended(0xED) Opcode not implemented: {:x}", op_code); }
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
        let mut clock = clocks::Clock::new();
        let mut memory = memory::MemoryAbsolute::new();
        let mut pc_state = pc_state::PcState::new();
        let mut ports = ports::Ports::new();
        let mut interuptor = interuptor::Interuptor::new();

        // ld_r_r instructions ( 0b01dddsss) 
        // 111 -> A, 000 -> B, 001 -> C, 
        // 010 -> D, 011 -> E, 100 -> H, 
        // 101 -> L
        
        assert_eq!(clock.cycles, 0);
        assert_eq!(pc_state.get_b(), 0);

        pc_state.set_c(0x42);
        instructions::Instruction::execute(0b01000001, &mut clock, &mut memory, &mut pc_state, &mut ports, &mut interuptor); // LD r,'r  C -> B
        assert_eq!(pc_state.get_b(), 0x42);
        assert_eq!(clock.cycles, 4);
    }

    #[test]
    fn test_jump_functions() {
        let mut clock = clocks::Clock::new();
        let mut memory = memory::MemoryAbsolute::new();
        let mut pc_state = pc_state::PcState::new();
        let mut ports = ports::Ports::new();
        let mut interuptor = interuptor::Interuptor::new();

        pc_state.set_hl(0x4233);
        pc_state.set_pc(0x2003);
        instructions::Instruction::execute(0xE9, &mut clock, &mut memory, &mut pc_state, &mut ports, &mut interuptor); // JP (HL)
        assert_eq!(pc_state.get_pc(), 0x4233);
    }
}
