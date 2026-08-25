use std::ffi::{CStr, c_char, c_int, c_long, c_uint, c_void};
use std::sync::Once;
pub(crate) type CurlCode = c_uint;
pub(crate) type CurlMCode = c_uint;
pub(crate) type CurlOption = c_uint;
pub(crate) type CurlInfo = c_uint;
pub(crate) enum Curl {}
pub(crate) enum CurlSlist {}
pub(crate) enum CurlMulti {}
#[repr(C)]
pub(crate) union CurlMsgData {
    pub(crate) result: CurlCode,
    _pointer: *mut c_void,
}
#[repr(C)]
pub(crate) struct CurlMsg {
    pub(crate) message: c_int,
    pub(crate) easy_handle: *mut Curl,
    pub(crate) data: CurlMsgData,
}
pub(crate) const CURLE_OK: CurlCode = 0;
pub(crate) const CURLM_OK: CurlMCode = 0;
pub(crate) const CURLMSG_DONE: c_int = 1;
const OBJECT: CurlOption = 10_000;
const FUNCTION: CurlOption = 20_000;
pub(crate) const CURLOPT_URL: CurlOption = OBJECT + 2;
pub(crate) const CURLOPT_PROXY: CurlOption = OBJECT + 4;
pub(crate) const CURLOPT_POSTFIELDS: CurlOption = OBJECT + 15;
pub(crate) const CURLOPT_HTTPHEADER: CurlOption = OBJECT + 23;
pub(crate) const CURLOPT_HEADERDATA: CurlOption = OBJECT + 29;
pub(crate) const CURLOPT_CUSTOMREQUEST: CurlOption = OBJECT + 36;
pub(crate) const CURLOPT_WRITEDATA: CurlOption = OBJECT + 1;
pub(crate) const CURLOPT_ACCEPT_ENCODING: CurlOption = OBJECT + 102;
pub(crate) const CURLOPT_WRITEFUNCTION: CurlOption = FUNCTION + 11;
pub(crate) const CURLOPT_HEADERFUNCTION: CurlOption = FUNCTION + 79;
pub(crate) const CURLOPT_NOBODY: CurlOption = 44;
pub(crate) const CURLOPT_FOLLOWLOCATION: CurlOption = 52;
pub(crate) const CURLOPT_POSTFIELDSIZE: CurlOption = 60;
pub(crate) const CURLOPT_PROXYTYPE: CurlOption = 101;
pub(crate) const CURLOPT_TIMEOUT_MS: CurlOption = 155;
pub(crate) const CURLOPT_CONNECTTIMEOUT_MS: CurlOption = 156;
pub(crate) const CURLINFO_EFFECTIVE_URL: CurlInfo = 0x100001;
pub(crate) const CURLINFO_RESPONSE_CODE: CurlInfo = 0x200002;
pub(crate) const CURLMOPT_SOCKETFUNCTION: c_uint = 20_001;
pub(crate) const CURLMOPT_SOCKETDATA: c_uint = 10_002;
pub(crate) const CURLMOPT_TIMERFUNCTION: c_uint = 20_004;
pub(crate) const CURLMOPT_TIMERDATA: c_uint = 10_005;
static INIT: Once = Once::new();
unsafe extern "C" {
    fn curl_global_init(flags: c_long) -> CurlCode;
    pub(crate) fn curl_easy_init() -> *mut Curl;
    pub(crate) fn curl_easy_cleanup(handle: *mut Curl);
    pub(crate) fn curl_easy_reset(handle: *mut Curl);
    fn curl_easy_setopt(handle: *mut Curl, option: CurlOption, ...) -> CurlCode;
    fn curl_easy_getinfo(handle: *mut Curl, info: CurlInfo, ...) -> CurlCode;
    pub(crate) fn curl_easy_impersonate(
        handle: *mut Curl,
        target: *const c_char,
        default_headers: c_int,
    ) -> CurlCode;
    pub(crate) fn curl_easy_strerror(code: CurlCode) -> *const c_char;
    pub(crate) fn curl_slist_append(list: *mut CurlSlist, value: *const c_char) -> *mut CurlSlist;
    pub(crate) fn curl_slist_free_all(list: *mut CurlSlist);
    pub(crate) fn curl_multi_init() -> *mut CurlMulti;
    pub(crate) fn curl_multi_add_handle(multi: *mut CurlMulti, easy: *mut Curl) -> CurlMCode;
    pub(crate) fn curl_multi_remove_handle(multi: *mut CurlMulti, easy: *mut Curl) -> CurlMCode;
    pub(crate) fn curl_multi_socket_action(
        multi: *mut CurlMulti,
        socket: c_int,
        events: c_int,
        running: *mut c_int,
    ) -> CurlMCode;
    pub(crate) fn curl_multi_info_read(multi: *mut CurlMulti, queued: *mut c_int) -> *mut CurlMsg;
    pub(crate) fn curl_multi_setopt(multi: *mut CurlMulti, option: c_uint, ...) -> CurlMCode;
    pub(crate) fn curl_multi_cleanup(multi: *mut CurlMulti) -> CurlMCode;
}
pub(crate) fn global_init() {
    INIT.call_once(|| unsafe {
        let _ = curl_global_init(3);
    });
}
pub(crate) unsafe fn setopt<T>(
    handle: *mut Curl,
    option: CurlOption,
    value: T,
) -> Result<(), String> {
    let code = unsafe { curl_easy_setopt(handle, option, value) };
    if code == CURLE_OK {
        Ok(())
    } else {
        Err(error(code))
    }
}
pub(crate) unsafe fn getinfo<T>(
    handle: *mut Curl,
    info: CurlInfo,
    output: *mut T,
) -> Result<(), String> {
    let code = unsafe { curl_easy_getinfo(handle, info, output) };
    if code == CURLE_OK {
        Ok(())
    } else {
        Err(error(code))
    }
}
pub(crate) fn error(code: CurlCode) -> String {
    unsafe {
        CStr::from_ptr(curl_easy_strerror(code))
            .to_string_lossy()
            .into_owned()
    }
}
