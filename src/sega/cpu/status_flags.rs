use super::pc_state;

pub fn signed_char_to_int(v: i8) -> i16 {
    return v as i16;
}

// Calculate the parity.  Check if the the number of bits is odd or even.
// Even number of '1's -> return 'true' (P=1)
// Odd number of '1's -> return 'false' (P=0)
pub fn calculate_parity(a: u8) -> bool {
  // Do parity half at a time
  let mut h = (a >> 4) ^ (a & 0xF);
  h = (h >> 2) ^ (h & 0x3);
  (h >> 1) == (h & 0x1) // Even parity
}

// self.pc_state.Add two 8 bit ints plus the carry bit, and set flags accordingly
pub fn u8_carry(a:u8, b:u8, c:bool, f_status: &mut pc_state::PcStatusFlagFields) -> u8 {
    let r = a.wrapping_add(b).wrapping_add(u8::from(c));

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

    zero_and_sign_flags(f_status, r);
    
    return r;
}

pub fn i8_carry(a:u8, b:u8, c:bool, f_status: &mut pc_state::PcStatusFlagFields) -> u8 {

    let mut r = (a as i16).wrapping_sub((b as i8) as i16).wrapping_sub(c as i16) as u16;
    let rc = (a as i16).wrapping_sub((b as i8) as i16).wrapping_sub(c as i16) as u8;
    let hr = ((a & 0xF) as u8).wrapping_sub((b & 0xF) as u8).wrapping_sub(c as u8);

    if 0 != (rc & 0x80) {
        f_status.set_s(1);
    } else {
        f_status.set_s(0);
    }

    if r == 0 {    // result zero
        f_status.set_z(1);    // result zero
    } else {
        f_status.set_z(0);    // result zero
    }
    if 0 != (hr & 0x10) {
        f_status.set_h(1);
    } else {
        f_status.set_h(0);
    }

    // overflow
    r = ((a as i16).wrapping_sub((b as i8) as i16).wrapping_sub(c as i16) as u16) & 0xFFF;
    if ((r & 0x180) != 0) && 
       ((r & 0x180) != 0x180) { // Overflow
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }

    f_status.set_n(1);

    r =  ((a as i16) & 0xFF).wrapping_sub((b as i8) as i16).wrapping_sub(c as i16) as u16;
    if 0 != (r & 0x100) { // cpu_state->Borrow (?) 
        f_status.set_c(1); // cpu_state->Borrow (?) 
    } else {
        f_status.set_c(0); // cpu_state->Borrow (?) 
    }

    r as u8
}
// calculate the 'sub c' flags (although carry isn't used), this matches a
// previous implementation (to make comparisons easier). 
// TODO: Once other issues are sorted out, revisit setting of these flags.
pub fn i8_no_carry(a:u8, b:u8, f_status: &mut pc_state::PcStatusFlagFields) -> u8 {

    let mut r  = ((signed_char_to_int(a as i8) - signed_char_to_int(b as i8)) as u16) & 0xFFFF;
    let rc = (signed_char_to_int(a as i8) - signed_char_to_int(b as i8)) & 0xFF;
    let hr  = ((signed_char_to_int(a as i8) & 0xF) - (signed_char_to_int(b as i8) & 0xF)) as u8;

    if 0 != (rc & 0x80) {
        f_status.set_s(1);
    } else {
        f_status.set_s(0);
    }

    if r == 0 {    // result zero
        f_status.set_z(1);    // result zero
    } else {
        f_status.set_z(0);    // result zero
    }
    if 0 != (hr & 0x10) {
        f_status.set_h(1);
    } else {
        f_status.set_h(0);
    }

    // overflow
    r = ((signed_char_to_int(a as i8) - signed_char_to_int(b as i8)) as u16) & 0xFFF;
    if ((r & 0x180) != 0) && 
       ((r & 0x180) != 0x180) { // Overflow
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }

    f_status.set_n(1);

    r  = (((a as i16) & 0xFF) - ((b as i16) & 0xFF)) as u16;
    if 0 != (r & 0x100) { // cpu_state->Borrow (?) 
        f_status.set_c(1); // cpu_state->Borrow (?) 
    } else {
        f_status.set_c(0); // cpu_state->Borrow (?) 
    }

    r as u8
}

pub fn u16_carry(a:u16, b:u16, c:bool, f_status: &mut pc_state::PcStatusFlagFields) -> u16 {
    // Perform a u16-bit add with carry, setting the flags (except N, which is
    // left to add/sub)
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
 
    return r;
}

pub fn u16_no_carry(a:u16, b:u16, f_status: &mut pc_state::PcStatusFlagFields) -> u16 {
    // Perform a u16-bit add with carry, setting the flags (except N, which is
    // left to add/sub)
    let r = a.wrapping_add(b);

    f_status.set_h(0);
    if (((a & 0xFFF) + (b & 0xFFF)) & 0x1000) == 0x1000 { // Half carry
        f_status.set_h(1);
    } else {
        f_status.set_h(0);
    }
 
    if a >= 0xFFFF - b {
        f_status.set_c(1);
    } else {
        f_status.set_c(0);
    }
 
    return r;
}

pub fn calculate_dec_flags(status: &mut pc_state::PcStatusFlagFields, new_value: u8) {
    status.set_n(1);
    if (new_value & 0xF) == 0xF { // Half borrow
      status.set_h(1);
    } else {
      status.set_h(0);
    }

    if new_value == 0x7F { // Was 80
      status.set_pv(1);
    } else {
      status.set_pv(0);
    }

    status.set_n(1);
    zero_and_sign_flags(status, new_value);
}

pub fn calculate_inc_flags(status: &mut pc_state::PcStatusFlagFields, new_value: u8) {
    if (new_value & 0xF) == 0x0 { // Half borrow
      status.set_h(1);
    } else {
      status.set_h(0);
    }

    if new_value == 0x80 { // Was 0x7F
      status.set_pv(1);
    } else {
      status.set_pv(0);
    }

    status.set_n(0);
    zero_and_sign_flags(status, new_value);
}

pub fn accumulator_flags(status: &mut pc_state::PcStatusFlagFields, accumulator: u8, iff2: bool) {
    // Used by LD A, I; LD A, R
      status.set_n(0);
      status.set_h(0);
      status.set_pv(iff2 as u8);
      zero_and_sign_flags(status, accumulator)
}

pub fn and_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
      // Used by AND s
      status.set_c(0);
      status.set_n(0);
      status.set_h(1);
      status.set_pv(calculate_parity(value) as u8); // Documented as 'set on overflow', not sure what it should be here
      zero_and_sign_flags(status, value)
}

pub fn xor_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
      // Used by AND s
      status.set_c(0);
      status.set_n(0);
      status.set_h(0);
      status.set_pv(calculate_parity(value) as u8); // Documented as set on even for xor
      zero_and_sign_flags(status, value)
}

// The 'new' value and carry
pub fn set_rotate_accumulator_flags(carry:bool, status: &mut pc_state::PcStatusFlagFields) -> () {
    status.set_c(carry as u8);
    status.set_h(0);
    status.set_n(0);
}

// The 'new' value and carry.  The flags set for rotating accumulator vs registers differ.
pub fn set_shift_register_flags(value: u8, carry:bool, status: &mut pc_state::PcStatusFlagFields) -> () {
    status.set_c(carry as u8);
    status.set_n(0);
    status.set_h(0);
    status.set_pv(calculate_parity(value) as u8); // Documented as set on even for xor
    zero_and_sign_flags(status, value)
}

pub fn rotate_decimal_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    // Carry not affected
    status.set_n(0);
    status.set_h(0);
    status.set_pv(calculate_parity(value) as u8);
    zero_and_sign_flags(status, value)
}

pub fn zero_and_sign_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    // Utility function, to set the zero and sign flags
      status.set_s((value & 0x80) >> 7);
      if value == 0 {
          status.set_z(1);
      } else {
          status.set_z(0);
      }
}

pub fn or_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    xor_flags(status, value);
}

// Add two 8 bit ints plus the carry bit, and set flags accordingly
pub fn set_bit_test_flags(r: u8, bit_pos: u8, f_status: &mut pc_state::PcStatusFlagFields) -> () {
    let bit = (r >> (bit_pos & 7)) & 0x1;
    f_status.set_z(bit ^ 0x1);
    f_status.set_pv(calculate_parity(bit) as u8); // Documented as 'unknown', not sure if/where this is needed.
    f_status.set_h(1);
    f_status.set_n(0);
    f_status.set_s(0);
}

#[cfg(test)]
mod tests {
    use crate::sega::cpu::pc_state;
    use crate::sega::cpu::status_flags;

    #[test]
    fn test_parity() {
        assert_eq!(status_flags::calculate_parity(0b11001001), true);
        assert_eq!(status_flags::calculate_parity(0b00101000), true);
        assert_eq!(status_flags::calculate_parity(0b10101001), true);
        assert_eq!(status_flags::calculate_parity(0b00101001), false);
        assert_eq!(status_flags::calculate_parity(0b00000001), false);
        assert_eq!(status_flags::calculate_parity(0b10000000), false);
    }
    #[test]
    fn test_bit_set() {
        let mut f_status = pc_state::PcStatusFlagFields(0);
        status_flags::set_bit_test_flags(0x30, 5, &mut f_status);
        assert_eq!(f_status.get_z(), 0);
        status_flags::set_bit_test_flags(0x30, 3, &mut f_status);
        assert_eq!(f_status.get_z(), 1);
    }
}
