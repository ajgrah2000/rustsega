use super::pc_state;

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
pub fn u8_carry(pc_state: &mut pc_state::PcState, a:u8, b:u8, c:bool) -> u8 {
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

pub fn u16_carry(pc_state: &mut pc_state::PcState, a:u16, b:u16, c:bool) -> u16 {
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

pub fn calculate_dec_flags(status: &mut pc_state::PcStatusFlagFields, new_value: u8) {
    if new_value & 0x80 != 0 { // Is negative
      status.set_s(1);
    } else {
      status.set_s(0);
    }
    if new_value == 0x0 { // Is zero
      status.set_z(1);
    } else {
      status.set_z(0);
    }
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
}

pub fn calculate_inc_flags(status: &mut pc_state::PcStatusFlagFields, new_value: u8) {
    if new_value & 0x80 != 0 { // Is negative
      status.set_s(1);
    } else {
      status.set_s(0);
    }
    if new_value == 0 { // Is zero
      status.set_z(1);
    } else {
      status.set_z(0);
    }
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
}

#[cfg(test)]
mod tests {
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
}
