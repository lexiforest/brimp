//! Owned keys and buffers from the BoringSSL instance embedded in curl-impersonate.
//! The C interface is declared in that SDK's curl-impersonate/webcrypto.h.
use super::{Result, error, require};
use std::{
    ffi::{c_int, c_uint, c_void},
    ptr::NonNull,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Slice {
    data: *const u8,
    len: usize,
}
impl From<&[u8]> for Slice {
    fn from(bytes: &[u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}
#[repr(C)]
struct Buffer {
    data: *mut u8,
    len: usize,
}
impl Drop for Buffer {
    fn drop(&mut self) {
        // SAFETY: the allocation came from this same BoringSSL instance.
        unsafe {
            bssl_webcrypto_buffer_free(Buffer {
                data: self.data,
                len: self.len,
            });
        }
    }
}
fn output(f: impl FnOnce(*mut Buffer) -> c_int) -> Result<Vec<u8>> {
    let mut buffer = Buffer {
        data: std::ptr::null_mut(),
        len: 0,
    };
    check(f(&mut buffer))?;
    if buffer.len == 0 {
        return Ok(Vec::new());
    }
    require(
        !buffer.data.is_null(),
        "OperationError",
        "Missing crypto output",
    )?;
    // SAFETY: successful native calls return an allocation of buffer.len bytes;
    // the guard releases it after this copy, including during unwinding.
    Ok(unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) }.to_vec())
}
fn check(code: c_int) -> Result<()> {
    require(
        code == 1,
        "OperationError",
        "Cryptographic operation failed",
    )
}
pub(super) fn hash_id(name: &str) -> Result<c_int> {
    match name {
        "SHA-1" => Ok(1),
        "SHA-256" => Ok(256),
        "SHA-384" => Ok(384),
        "SHA-512" => Ok(512),
        _ => Err(error("NotSupportedError", "Unsupported hash")),
    }
}
pub(super) fn curve_id(name: &str) -> Result<c_int> {
    match name {
        "P-256" => Ok(256),
        "P-384" => Ok(384),
        "P-521" => Ok(521),
        _ => Err(error("NotSupportedError", "Unsupported named curve")),
    }
}
pub(super) fn digest(hash: &str, data: &[u8]) -> Result<Vec<u8>> {
    let hash = hash_id(hash)?;
    output(|out| unsafe { bssl_webcrypto_digest(hash, data.into(), out) })
}
pub(super) fn hmac(hash: &str, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let hash = hash_id(hash)?;
    output(|out| unsafe { bssl_webcrypto_hmac(hash, key.into(), data.into(), out) })
}
pub(super) fn equal(a: &[u8], b: &[u8]) -> bool {
    unsafe { bssl_webcrypto_equal(a.into(), b.into()) == 1 }
}
pub(super) fn derive(
    algorithm: c_int,
    hash: &str,
    key: &[u8],
    salt: &[u8],
    info: &[u8],
    iterations: u32,
    out: &mut [u8],
) -> Result<()> {
    check(unsafe {
        bssl_webcrypto_derive(
            algorithm,
            hash_id(hash)?,
            key.into(),
            salt.into(),
            info.into(),
            iterations,
            out.as_mut_ptr(),
            out.len(),
        )
    })
}
pub(super) fn cipher(
    algorithm: c_int,
    encrypt: bool,
    key: &[u8],
    iv: &[u8],
    aad: &[u8],
    data: &[u8],
    tag_len: usize,
) -> Result<Vec<u8>> {
    output(|out| unsafe {
        bssl_webcrypto_cipher(
            algorithm,
            encrypt.into(),
            key.into(),
            iv.into(),
            aad.into(),
            data.into(),
            tag_len,
            out,
        )
    })
}

pub(super) struct NativeKey(NonNull<c_void>);
impl Drop for NativeKey {
    fn drop(&mut self) {
        unsafe {
            bssl_webcrypto_key_free(self.0.as_ptr());
        }
    }
}
impl NativeKey {
    fn new(ptr: *mut c_void) -> Result<Self> {
        NonNull::new(ptr)
            .map(Self)
            .ok_or_else(|| error("OperationError", "Invalid cryptographic key"))
    }
    pub(super) fn generate_rsa(bits: u32, exponent: &[u8]) -> Result<Self> {
        Self::new(unsafe { bssl_webcrypto_key_generate(1, 0, bits, exponent.into()) })
    }
    pub(super) fn generate_ec(curve: &str) -> Result<Self> {
        Self::new(unsafe { bssl_webcrypto_key_generate(2, curve_id(curve)?, 0, (&[][..]).into()) })
    }
    pub(super) fn parse(private: bool, der: &[u8]) -> Result<Self> {
        Self::new(unsafe { bssl_webcrypto_key_parse(private.into(), der.into()) })
    }
    pub(super) fn rsa(parts: &[Vec<u8>; 8]) -> Result<Self> {
        let slices = parts.each_ref().map(|part| part.as_slice().into());
        Self::new(unsafe { bssl_webcrypto_key_rsa(slices.as_ptr()) })
    }
    pub(super) fn ec(curve: &str, x: &[u8], y: &[u8], d: &[u8]) -> Result<Self> {
        Self::new(unsafe { bssl_webcrypto_key_ec(curve_id(curve)?, x.into(), y.into(), d.into()) })
    }
    pub(super) fn kind(&self) -> c_int {
        unsafe { bssl_webcrypto_key_kind(self.0.as_ptr()) }
    }
    pub(super) fn curve(&self) -> c_int {
        unsafe { bssl_webcrypto_key_curve(self.0.as_ptr()) }
    }
    pub(super) fn bits(&self) -> u32 {
        unsafe { bssl_webcrypto_key_bits(self.0.as_ptr()) }
    }
    pub(super) fn export(&self, format: c_int) -> Result<Vec<u8>> {
        output(|out| unsafe { bssl_webcrypto_key_export(self.0.as_ptr(), format, out) })
    }
    pub(super) fn sign(
        &self,
        scheme: c_int,
        hash: &str,
        salt: i32,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let hash = hash_id(hash)?;
        output(|out| unsafe {
            bssl_webcrypto_sign(self.0.as_ptr(), scheme, hash, salt, data.into(), out)
        })
    }
    pub(super) fn verify(
        &self,
        scheme: c_int,
        hash: &str,
        salt: i32,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        Ok(unsafe {
            bssl_webcrypto_verify(
                self.0.as_ptr(),
                scheme,
                hash_id(hash)?,
                salt,
                data.into(),
                signature.into(),
            ) == 1
        })
    }
    pub(super) fn oaep(
        &self,
        encrypt: bool,
        hash: &str,
        label: &[u8],
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let hash = hash_id(hash)?;
        output(|out| unsafe {
            bssl_webcrypto_oaep(
                self.0.as_ptr(),
                encrypt.into(),
                hash,
                label.into(),
                data.into(),
                out,
            )
        })
    }
    pub(super) fn ecdh(&self, peer: &Self) -> Result<Vec<u8>> {
        output(|out| unsafe { bssl_webcrypto_ecdh(self.0.as_ptr(), peer.0.as_ptr(), out) })
    }
}

// SAFETY contract: all callers pass live owned handles and borrowed slices for
// the duration of a synchronous call. Native code never retains Rust pointers.
// The runtime's brimp-net dependency links the library that owns these symbols.
unsafe extern "C" {
    fn bssl_webcrypto_buffer_free(buffer: Buffer);
    fn bssl_webcrypto_key_free(key: *mut c_void);
    fn bssl_webcrypto_digest(hash: c_int, data: Slice, out: *mut Buffer) -> c_int;
    fn bssl_webcrypto_hmac(hash: c_int, key: Slice, data: Slice, out: *mut Buffer) -> c_int;
    fn bssl_webcrypto_equal(a: Slice, b: Slice) -> c_int;
    fn bssl_webcrypto_derive(
        algorithm: c_int,
        hash: c_int,
        key: Slice,
        salt: Slice,
        info: Slice,
        iterations: u32,
        out: *mut u8,
        out_len: usize,
    ) -> c_int;
    fn bssl_webcrypto_cipher(
        cipher: c_int,
        encrypt: c_int,
        key: Slice,
        iv: Slice,
        aad: Slice,
        data: Slice,
        tag_len: usize,
        out: *mut Buffer,
    ) -> c_int;
    fn bssl_webcrypto_key_generate(
        kind: c_int,
        curve: c_int,
        bits: c_uint,
        exponent: Slice,
    ) -> *mut c_void;
    fn bssl_webcrypto_key_parse(private: c_int, der: Slice) -> *mut c_void;
    fn bssl_webcrypto_key_rsa(parts: *const Slice) -> *mut c_void;
    fn bssl_webcrypto_key_ec(curve: c_int, x: Slice, y: Slice, d: Slice) -> *mut c_void;
    fn bssl_webcrypto_key_kind(key: *const c_void) -> c_int;
    fn bssl_webcrypto_key_curve(key: *const c_void) -> c_int;
    fn bssl_webcrypto_key_bits(key: *const c_void) -> c_uint;
    fn bssl_webcrypto_key_export(key: *const c_void, format: c_int, out: *mut Buffer) -> c_int;
    fn bssl_webcrypto_sign(
        key: *const c_void,
        scheme: c_int,
        hash: c_int,
        salt: c_int,
        data: Slice,
        out: *mut Buffer,
    ) -> c_int;
    fn bssl_webcrypto_verify(
        key: *const c_void,
        scheme: c_int,
        hash: c_int,
        salt: c_int,
        data: Slice,
        signature: Slice,
    ) -> c_int;
    fn bssl_webcrypto_oaep(
        key: *const c_void,
        encrypt: c_int,
        hash: c_int,
        label: Slice,
        data: Slice,
        out: *mut Buffer,
    ) -> c_int;
    fn bssl_webcrypto_ecdh(key: *const c_void, peer: *const c_void, out: *mut Buffer) -> c_int;
}
