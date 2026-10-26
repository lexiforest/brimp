use crate::{Error, slice};
use std::ffi::c_void;

unsafe extern "C" {
    fn br_html_meta_charset(
        input: *const u8,
        length: usize,
        accept: unsafe extern "C" fn(*const u8, usize, *mut c_void) -> bool,
        context: *mut c_void,
    ) -> i32;
}

/// Find the first accepted charset declaration using Safari's token scan rules.
/// The predicate must not panic, since it is invoked through the C tokenizer.
pub fn html_meta_charset(input: &[u8], accepts: fn(&str) -> bool) -> Result<Option<String>, Error> {
    struct Scan {
        accepts: fn(&str) -> bool,
        label: Option<String>,
    }
    unsafe extern "C" fn accept(data: *const u8, length: usize, context: *mut c_void) -> bool {
        // SAFETY: the synchronous shim supplies live token data and our Scan.
        let scan = unsafe { &mut *context.cast::<Scan>() };
        let Ok(label) = std::str::from_utf8(unsafe { slice(data, length) }) else {
            return false;
        };
        if (scan.accepts)(label) {
            scan.label = Some(label.to_owned());
            true
        } else {
            false
        }
    }
    let mut scan = Scan {
        accepts,
        label: None,
    };
    // SAFETY: input and scan remain live for this synchronous call.
    let status = unsafe {
        br_html_meta_charset(
            input.as_ptr(),
            input.len(),
            accept,
            (&mut scan as *mut Scan).cast(),
        )
    };
    if status == 0 {
        Ok(scan.label)
    } else {
        Err(Error::Native)
    }
}
