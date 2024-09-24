use line_index::{LineIndex, WideEncoding};
use memchr::memmem;
use triomphe::Arc;

#[derive(Clone, Copy)]
pub enum PositionEncoding {
    Utf8,
    Wide(WideEncoding),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineEnding {
    Unix,
    Dos,
}

impl LineEnding {
    /// Replaces `\r\n` with `\n` in-place in `src`.
    pub fn normalize(src: String) -> (String, LineEnding) {
        // We replace `\r\n` with `\n` in-place, which doesn't break utf-8 encoding.
        // While we *can* call `as_mut_vec` and do surgery on the live string
        // directly, let's rather steal the contents of `src`. This makes the code
        // safe even if a panic occurs.

        let mut buf = src.into_bytes();
        let mut gap_len = 0;
        let mut tail = buf.as_mut_slice();
        let mut crlf_seen = false;

        let finder = memmem::Finder::new(b"\r\n");

        loop {
            let idx = match finder.find(&tail[gap_len..]) {
                None if crlf_seen => tail.len(),
                // SAFETY: buf is unchanged and therefore still contains utf8 data
                None => return (unsafe { String::from_utf8_unchecked(buf) }, LineEnding::Unix),
                Some(idx) => {
                    crlf_seen = true;
                    idx + gap_len
                }
            };
            tail.copy_within(gap_len..idx, 0);
            tail = &mut tail[idx - gap_len..];
            if tail.len() == gap_len {
                break;
            }
            gap_len += 1;
        }

        // Account for removed `\r`.
        // After `set_len`, `buf` is guaranteed to contain utf-8 again.
        let src = unsafe {
            let new_len = buf.len() - gap_len;
            buf.set_len(new_len);
            String::from_utf8_unchecked(buf)
        };
        (src, LineEnding::Dos)
    }
}

pub struct LineInfo {
    pub index: Arc<LineIndex>,
    pub ending: LineEnding,
    pub encoding: PositionEncoding,
}
