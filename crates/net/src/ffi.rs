use std::ffi::{CStr, c_char, c_int, c_long, c_uint, c_void};
use std::sync::Once;

use crate::error::Error;

/// libcurl info selector type.
pub(crate) type CurlInfo = c_uint;
/// libcurl option selector type.
pub(crate) type CurlOption = c_uint;
/// libcurl result code type.
pub(crate) type CurlCode = c_uint;
/// libcurl multi result code type.
pub(crate) type CurlMCode = c_uint;

/// Opaque curl easy handle.
pub(crate) enum Curl {}
/// Opaque curl linked-list handle for request headers.
pub(crate) enum CurlSlist {}
/// Opaque curl multi handle.
pub(crate) enum CurlMulti {}

#[repr(C)]
pub(crate) union CurlMsgData {
    pub(crate) whatever: *mut c_void,
    pub(crate) result: CurlCode,
}

#[repr(C)]
pub(crate) struct CurlMsg {
    pub(crate) message: c_int,
    pub(crate) easy_handle: *mut Curl,
    pub(crate) data: CurlMsgData,
}

pub(crate) const CURLE_OK: CurlCode = 0;
pub(crate) const CURLM_OK: CurlMCode = 0;
pub(crate) const CURLM_CALL_MULTI_PERFORM: CurlMCode = u32::MAX;
pub(crate) const CURLMSG_DONE: c_int = 1;
pub(crate) const CURL_GLOBAL_DEFAULT: c_long = 3;
pub(crate) const CURLPIPE_MULTIPLEX: c_long = 2;
pub(crate) const CURLMOPT_PIPELINING: c_uint = 3;
pub(crate) const CURL_WRITEFUNC_PAUSE: usize = 0x10000001;
pub(crate) const CURLPAUSE_CONT: c_int = 0;
pub(crate) const CURLOPT_NOSIGNAL: CurlOption = 99;
pub(crate) const CURLOPT_SUPPRESS_CONNECT_HEADERS: CurlOption = 265;
pub(crate) const CURLOPTTYPE_LONG: CurlOption = 0;
pub(crate) const CURLOPTTYPE_OBJECTPOINT: CurlOption = 10_000;
pub(crate) const CURLOPTTYPE_FUNCTIONPOINT: CurlOption = 20_000;
pub(crate) const CURLOPT_FILE: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1;
pub(crate) const CURLOPT_WRITEHEADER: CurlOption = CURLOPTTYPE_OBJECTPOINT + 29;
pub(crate) const CURLOPT_URL: CurlOption = CURLOPTTYPE_OBJECTPOINT + 2;
pub(crate) const CURLOPT_PROXY: CurlOption = CURLOPTTYPE_OBJECTPOINT + 4;
pub(crate) const CURLOPT_WRITEFUNCTION: CurlOption = CURLOPTTYPE_FUNCTIONPOINT + 11;
pub(crate) const CURLOPT_POSTFIELDS: CurlOption = CURLOPTTYPE_OBJECTPOINT + 15;
pub(crate) const CURLOPT_HTTPHEADER: CurlOption = CURLOPTTYPE_OBJECTPOINT + 23;
pub(crate) const CURLOPT_CUSTOMREQUEST: CurlOption = CURLOPTTYPE_OBJECTPOINT + 36;
pub(crate) const CURLOPT_NOBODY: CurlOption = CURLOPTTYPE_LONG + 44;
pub(crate) const CURLOPT_FOLLOWLOCATION: CurlOption = CURLOPTTYPE_LONG + 52;
pub(crate) const CURLOPT_POSTFIELDSIZE: CurlOption = CURLOPTTYPE_LONG + 60;
pub(crate) const CURLOPT_PROXYTYPE: CurlOption = CURLOPTTYPE_LONG + 101;
pub(crate) const CURLOPT_HEADERFUNCTION: CurlOption = CURLOPTTYPE_FUNCTIONPOINT + 79;
pub(crate) const CURLOPT_ACCEPT_ENCODING: CurlOption = CURLOPTTYPE_OBJECTPOINT + 102;
pub(crate) const CURLOPT_SSLVERSION: CurlOption = CURLOPTTYPE_LONG + 32;
pub(crate) const CURLOPT_SSL_CIPHER_LIST: CurlOption = CURLOPTTYPE_OBJECTPOINT + 83;
pub(crate) const CURLOPT_HTTP_VERSION: CurlOption = CURLOPTTYPE_LONG + 84;
pub(crate) const CURLOPT_TIMEOUT_MS: CurlOption = CURLOPTTYPE_LONG + 155;
pub(crate) const CURLOPT_CONNECTTIMEOUT_MS: CurlOption = CURLOPTTYPE_LONG + 156;
pub(crate) const CURLOPT_SSL_ENABLE_ALPN: CurlOption = CURLOPTTYPE_LONG + 226;
pub(crate) const CURLOPT_STREAM_WEIGHT: CurlOption = CURLOPTTYPE_LONG + 239;
pub(crate) const CURLOPT_SSL_EC_CURVES: CurlOption = CURLOPTTYPE_OBJECTPOINT + 298;
pub(crate) const CURLOPT_ECH: CurlOption = CURLOPTTYPE_OBJECTPOINT + 325;
pub(crate) const CURLOPT_SSL_SIG_HASH_ALGS: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1001;
pub(crate) const CURLOPT_SSL_ENABLE_ALPS: CurlOption = CURLOPTTYPE_LONG + 1002;
pub(crate) const CURLOPT_SSL_CERT_COMPRESSION: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1003;
pub(crate) const CURLOPT_SSL_ENABLE_TICKET: CurlOption = CURLOPTTYPE_LONG + 1004;
pub(crate) const CURLOPT_HTTP2_PSEUDO_HEADERS_ORDER: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1005;
pub(crate) const CURLOPT_HTTP2_SETTINGS: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1006;
pub(crate) const CURLOPT_SSL_PERMUTE_EXTENSIONS: CurlOption = CURLOPTTYPE_LONG + 1007;
pub(crate) const CURLOPT_HTTP2_WINDOW_UPDATE: CurlOption = CURLOPTTYPE_LONG + 1008;
pub(crate) const CURLOPT_TLS_GREASE: CurlOption = CURLOPTTYPE_LONG + 1011;
pub(crate) const CURLOPT_TLS_EXTENSION_ORDER: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1012;
pub(crate) const CURLOPT_STREAM_EXCLUSIVE: CurlOption = CURLOPTTYPE_LONG + 1013;
pub(crate) const CURLOPT_TLS_SIGNED_CERT_TIMESTAMPS: CurlOption = CURLOPTTYPE_LONG + 1015;
pub(crate) const CURLOPT_TLS_STATUS_REQUEST: CurlOption = CURLOPTTYPE_LONG + 1016;
pub(crate) const CURLOPT_TLS_DELEGATED_CREDENTIALS: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1017;
pub(crate) const CURLOPT_TLS_RECORD_SIZE_LIMIT: CurlOption = CURLOPTTYPE_LONG + 1018;
pub(crate) const CURLOPT_TLS_KEY_SHARES_LIMIT: CurlOption = CURLOPTTYPE_LONG + 1019;
pub(crate) const CURLOPT_TLS_USE_NEW_ALPS_CODEPOINT: CurlOption = CURLOPTTYPE_LONG + 1020;
pub(crate) const CURLOPT_HTTP2_NO_PRIORITY: CurlOption = CURLOPTTYPE_LONG + 1021;
pub(crate) const CURLOPT_SPLIT_COOKIES: CurlOption = CURLOPTTYPE_LONG + 1023;
pub(crate) const CURLOPT_FORM_BOUNDARY: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1024;
pub(crate) const CURLOPT_HTTPHEADER_ORDER: CurlOption = CURLOPTTYPE_OBJECTPOINT + 1030;
pub(crate) const CURLOPT_WRITEDATA: CurlOption = CURLOPT_FILE;
pub(crate) const CURLOPT_HEADERDATA: CurlOption = CURLOPT_WRITEHEADER;
pub(crate) const CURLINFO_STRING: CurlInfo = 0x100000;
pub(crate) const CURLINFO_LONG: CurlInfo = 0x200000;
pub(crate) const CURLINFO_EFFECTIVE_URL: CurlInfo = CURLINFO_STRING + 1;
pub(crate) const CURLINFO_RESPONSE_CODE: CurlInfo = CURLINFO_LONG + 2;
pub(crate) const CURLINFO_PRIMARY_IP: CurlInfo = CURLINFO_STRING + 32;
pub(crate) const CURLINFO_NUM_CONNECTS: CurlInfo = CURLINFO_LONG + 26;
pub(crate) const CURLINFO_PRIMARY_PORT: CurlInfo = CURLINFO_LONG + 40;
pub(crate) const CURLINFO_LOCAL_IP: CurlInfo = CURLINFO_STRING + 41;
pub(crate) const CURLINFO_LOCAL_PORT: CurlInfo = CURLINFO_LONG + 42;
pub(crate) const CURLINFO_HTTP_VERSION: CurlInfo = CURLINFO_LONG + 46;

static CURL_GLOBAL_INIT: Once = Once::new();

unsafe extern "C" {
    /// Initializes libcurl process-global state.
    pub(crate) fn curl_global_init(flags: c_long) -> CurlCode;
    /// Allocates a curl easy handle.
    pub(crate) fn curl_easy_init() -> *mut Curl;
    /// Frees a curl easy handle.
    pub(crate) fn curl_easy_cleanup(curl: *mut Curl);
    /// Resets request-specific options while retaining the handle's live connections.
    pub(crate) fn curl_easy_reset(curl: *mut Curl);
    /// Executes the configured transfer.
    pub(crate) fn curl_easy_perform(curl: *mut Curl) -> CurlCode;
    pub(crate) fn curl_easy_pause(curl: *mut Curl, flags: c_int) -> CurlCode;
    /// Returns a static error string for a libcurl result code.
    pub(crate) fn curl_easy_strerror(code: CurlCode) -> *const c_char;
    /// Sets an option on a curl easy handle.
    pub(crate) fn curl_easy_setopt(curl: *mut Curl, option: CurlOption, ...) -> CurlCode;
    /// Reads information from a completed curl easy transfer.
    pub(crate) fn curl_easy_getinfo(curl: *mut Curl, info: CurlInfo, ...) -> CurlCode;
    /// Applies a libcurl-impersonate browser profile to a curl easy handle.
    pub(crate) fn curl_easy_impersonate(
        curl: *mut Curl,
        target: *const c_char,
        default_headers: c_int,
    ) -> CurlCode;
    /// Appends a string to a curl string list.
    pub(crate) fn curl_slist_append(list: *mut CurlSlist, string: *const c_char) -> *mut CurlSlist;
    /// Frees a curl string list.
    pub(crate) fn curl_slist_free_all(list: *mut CurlSlist);
    /// Allocates a multi handle and its shared connection cache.
    pub(crate) fn curl_multi_init() -> *mut CurlMulti;
    /// Adds an easy transfer to a multi handle.
    pub(crate) fn curl_multi_add_handle(multi: *mut CurlMulti, easy: *mut Curl) -> CurlMCode;
    /// Removes an easy transfer from a multi handle.
    pub(crate) fn curl_multi_remove_handle(multi: *mut CurlMulti, easy: *mut Curl) -> CurlMCode;
    /// Advances all active transfers without blocking.
    pub(crate) fn curl_multi_perform(multi: *mut CurlMulti, running: *mut c_int) -> CurlMCode;
    /// Waits for activity on any transfer in a multi handle.
    pub(crate) fn curl_multi_poll(
        multi: *mut CurlMulti,
        extra_fds: *mut c_void,
        extra_nfds: c_uint,
        timeout_ms: c_int,
        numfds: *mut c_int,
    ) -> CurlMCode;
    pub(crate) fn curl_multi_wakeup(multi: *mut CurlMulti) -> CurlMCode;
    /// Reads the next completed-transfer message.
    pub(crate) fn curl_multi_info_read(
        multi: *mut CurlMulti,
        messages_in_queue: *mut c_int,
    ) -> *mut CurlMsg;
    /// Sets a multi-handle option.
    pub(crate) fn curl_multi_setopt(multi: *mut CurlMulti, option: c_uint, ...) -> CurlMCode;
    /// Returns a static description for a multi result code.
    pub(crate) fn curl_multi_strerror(code: CurlMCode) -> *const c_char;
    /// Frees a multi handle and its connection cache.
    pub(crate) fn curl_multi_cleanup(multi: *mut CurlMulti) -> CurlMCode;
}

/// Initializes libcurl once for the current process.
pub(crate) fn global_init() {
    CURL_GLOBAL_INIT.call_once(|| unsafe {
        curl_global_init(CURL_GLOBAL_DEFAULT);
    });
}

/// Calls `curl_easy_setopt` and converts libcurl errors into [`Error`].
pub(crate) unsafe fn setopt<T>(
    handle: *mut Curl,
    option: CurlOption,
    value: T,
) -> Result<(), Error> {
    let result = unsafe { curl_easy_setopt(handle, option, value) };
    if result == CURLE_OK {
        Ok(())
    } else {
        Err(Error::Network(curl_error(handle, result)))
    }
}

/// Calls `curl_easy_getinfo` and converts libcurl errors into [`Error`].
pub(crate) unsafe fn getinfo<T>(
    handle: *mut Curl,
    option: CurlInfo,
    value: *mut T,
) -> Result<(), Error> {
    let result = unsafe { curl_easy_getinfo(handle, option, value) };
    if result == CURLE_OK {
        Ok(())
    } else {
        Err(Error::Network(curl_error(handle, result)))
    }
}

/// Formats a libcurl error code with the associated easy-handle pointer.
pub(crate) fn curl_error(handle: *mut Curl, code: CurlCode) -> String {
    unsafe {
        let message = curl_easy_strerror(code);
        if message.is_null() {
            return format!("curl error {code}");
        }
        let message = CStr::from_ptr(message).to_string_lossy();
        format!("{message} ({code}) on handle {handle:p}")
    }
}

/// Formats a libcurl multi-interface result code.
pub(crate) fn curl_multi_error(code: CurlMCode) -> String {
    unsafe {
        let message = curl_multi_strerror(code);
        if message.is_null() {
            return format!("curl multi error {code}");
        }
        format!("{} ({code})", CStr::from_ptr(message).to_string_lossy())
    }
}

pub(crate) const CURLOPT_CAINFO: CurlOption = CURLOPTTYPE_OBJECTPOINT + 65;
pub(crate) const CURLOPT_USERNAME: CurlOption = CURLOPTTYPE_OBJECTPOINT + 173;
pub(crate) const CURLOPT_PASSWORD: CurlOption = CURLOPTTYPE_OBJECTPOINT + 174;
pub(crate) const CURLOPT_PROXYUSERNAME: CurlOption = CURLOPTTYPE_OBJECTPOINT + 175;
pub(crate) const CURLOPT_PROXYPASSWORD: CurlOption = CURLOPTTYPE_OBJECTPOINT + 176;
pub(crate) const CURLOPT_HTTPAUTH: CurlOption = 107;
pub(crate) const CURLOPT_PROXYAUTH: CurlOption = 111;
pub(crate) const CURLINFO_HEADER_SIZE: CurlInfo = 0x20000b;
pub(crate) const CURLINFO_SIZE_UPLOAD_T: CurlInfo = 0x600007;
pub(crate) const CURLINFO_SIZE_DOWNLOAD_T: CurlInfo = 0x600008;

pub(crate) const CURLE_AGAIN: CurlCode = 81;
pub(crate) const CURLOPT_VERBOSE: CurlOption = 41;
pub(crate) const CURLOPT_DEBUGFUNCTION: CurlOption = CURLOPTTYPE_FUNCTIONPOINT + 94;
pub(crate) const CURLOPT_DEBUGDATA: CurlOption = CURLOPTTYPE_OBJECTPOINT + 95;
pub(crate) const CURLOPT_CONNECT_ONLY: CurlOption = 141;
pub(crate) const CURLWS_TEXT: u32 = 1;
pub(crate) const CURLWS_BINARY: u32 = 2;
pub(crate) const CURLWS_CLOSE: u32 = 8;
#[repr(C)]
pub(crate) struct CurlWsFrame {
    pub(crate) age: i32,
    pub(crate) flags: i32,
    pub(crate) offset: i64,
    pub(crate) bytesleft: i64,
    pub(crate) len: usize,
}
unsafe extern "C" {
    pub(crate) fn curl_ws_recv(
        handle: *mut Curl,
        buffer: *mut c_void,
        length: usize,
        received: *mut usize,
        metadata: *mut *const CurlWsFrame,
    ) -> CurlCode;
    pub(crate) fn curl_ws_send(
        handle: *mut Curl,
        buffer: *const c_void,
        length: usize,
        sent: *mut usize,
        frame_size: i64,
        flags: u32,
    ) -> CurlCode;
}
