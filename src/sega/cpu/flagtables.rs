// Calculate the parity.  Check if the the number of bits is odd or even.
// Even number of '1's -> return 'true' (P=1)
// Odd number of '1's -> return 'false' (P=0)
pub fn calculate_parity(a: u8) -> bool {
  // Do parity half at a time
  let mut h = (a >> 4) ^ (a & 0xF);
  h = (h >> 2) ^ (h & 0x3);
  (h >> 1) == (h & 0x1) // Even parity
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
