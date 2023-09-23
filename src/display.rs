/// Convert to a byte representation (CP-437)
pub trait DisplayBytes {
    fn display_bytes(&self) -> Vec<u8>;
}

impl<const N: usize> DisplayBytes for &[u8; N] {
    // This is provided as a convenience for use in the
    // bconcat macro. It unfortunately introduces an extra allocation
    fn display_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl DisplayBytes for &[u8] {
    // This is provided as a convenience for use in the
    // bconcat macro. It unfortunately introduces an extra allocation
    fn display_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl DisplayBytes for usize {
    /// Convert an integer to an ASCII string in decimal
    fn display_bytes(&self) -> Vec<u8> {
        let mut n = *self;
        let mut tmp = Vec::new();
        loop {
            tmp.insert(0, (n % 10) as u8 + b'0');
            n /= 10;
            if n == 0 {
                break;
            }
        }
        tmp
    }
}

impl DisplayBytes for u8 {
    /// Convert an integer to an ASCII string in decimal
    fn display_bytes(&self) -> Vec<u8> {
        (*self as usize).display_bytes()
    }
}

impl DisplayBytes for f64 {
    /// Convert a float to an ASCII string in decimal (truncates
    /// to whole part). Assumes float value is within range of
    /// usize!
    fn display_bytes(&self) -> Vec<u8> {
        (self.floor() as usize).display_bytes()
    }
}
