macro_rules! encode {
    ($number:expr, $buf:expr) => {{
        let mut n = $number;
        let mut i = 0;
        for b in $buf.iter_mut() {
            *b = n as u8 | 0x80;
            n >>= 7;
            if n == 0 {
                *b &= 0x7f;
                break
            }
            i += 1
        }
        debug_assert_eq!(n, 0);
        &$buf[0..=i]
    }}
}

pub fn u64(number: u64, buf: &mut [u8; U64_LEN]) -> &[u8] {
    encode!(number, buf)
}

const U64_LEN: usize = 10;
