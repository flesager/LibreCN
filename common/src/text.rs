//! Text in fixed-size message bodies.

use std::fmt;

/// Copies as much of `s` as fits in `dst`, cut on a character boundary.
/// Returns the bytes written.
pub(crate) fn copy_truncated(dst: &mut [u8], s: &str) -> usize {
    let mut n = s.len().min(dst.len());
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    dst[..n].copy_from_slice(&s.as_bytes()[..n]);
    n
}

/// `fmt::Write` into a fixed buffer; remembers whether something was cut.
pub(crate) struct FixedWriter<'a> {
    pub(crate) buf: &'a mut [u8],
    pub(crate) len: usize,
    pub(crate) truncated: bool,
}

impl fmt::Write for FixedWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let n = copy_truncated(&mut self.buf[self.len..], s);
        self.len += n;
        self.truncated |= n < s.len();
        Ok(())
    }
}

/// The valid UTF-8 prefix of `bytes` (bodies come from other EOs).
pub(crate) fn as_str(bytes: &[u8]) -> &str {
    match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => std::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::*;

    #[test]
    fn cuts_on_character_boundaries() {
        let mut buf = [0u8; 4];
        assert_eq!(copy_truncated(&mut buf, "aéé"), 3, "a + é, the second é does not fit");
        let mut w = FixedWriter { buf: &mut buf, len: 0, truncated: false };
        write!(w, "{}", 12345).unwrap();
        assert_eq!((w.len, w.truncated), (4, true));
        assert_eq!(as_str(&buf), "1234");
    }
}
