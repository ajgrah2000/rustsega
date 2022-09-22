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
    use crate::sega::cpu::flagtables;

    #[test]
    fn test_parity() {
        assert_eq!(flagtables::calculate_parity(0b11001001), true);
        assert_eq!(flagtables::calculate_parity(0b00101000), true);
        assert_eq!(flagtables::calculate_parity(0b10101001), true);
        assert_eq!(flagtables::calculate_parity(0b00101001), false);
        assert_eq!(flagtables::calculate_parity(0b00000001), false);
        assert_eq!(flagtables::calculate_parity(0b10000000), false);
    }
}
