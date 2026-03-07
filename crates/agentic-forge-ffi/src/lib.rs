//! FFI bindings for AgenticForge.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn agentic_forge_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
/// # Safety
/// Caller must provide valid, non-null, NUL-terminated C strings for all
/// pointer arguments. Returned pointer must be freed with
/// `agentic_forge_free_string`.
pub unsafe extern "C" fn agentic_forge_create_blueprint(
    name: *const c_char,
    description: *const c_char,
    domain: *const c_char,
) -> *mut c_char {
    if name.is_null() || description.is_null() || domain.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: pointers are checked for null above and are expected to be valid
    // null-terminated strings from the FFI boundary.
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    // SAFETY: same preconditions as above.
    let desc = unsafe { CStr::from_ptr(description) }.to_string_lossy();
    // SAFETY: same preconditions as above.
    let domain_str = unsafe { CStr::from_ptr(domain) }.to_string_lossy();

    let domain = agentic_forge_core::types::intent::Domain::from_name(&domain_str)
        .unwrap_or(agentic_forge_core::types::intent::Domain::Custom);

    let mut engine = agentic_forge_core::engine::ForgeEngine::new();
    match engine.create_blueprint(name.as_ref(), desc.as_ref(), domain) {
        Ok(id) => {
            let s = CString::new(id.to_string()).unwrap_or_default();
            s.into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
/// # Safety
/// `s` must be a pointer returned by `CString::into_raw` from this library and
/// must not be freed more than once.
pub unsafe extern "C" fn agentic_forge_free_string(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: `s` must come from `CString::into_raw` in this crate.
        unsafe { drop(CString::from_raw(s)) };
    }
}

#[no_mangle]
pub extern "C" fn agentic_forge_invention_count() -> u32 {
    agentic_forge_core::types::INVENTION_COUNT as u32
}

#[no_mangle]
pub extern "C" fn agentic_forge_tool_count() -> u32 {
    agentic_forge_core::types::MCP_TOOL_COUNT as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_version() {
        let version = agentic_forge_version();
        let s = unsafe { CStr::from_ptr(version) }.to_str().unwrap();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_invention_count() {
        assert_eq!(agentic_forge_invention_count(), 32);
    }

    #[test]
    fn test_tool_count() {
        assert_eq!(agentic_forge_tool_count(), 15);
    }

    #[test]
    fn test_create_blueprint() {
        let name = CString::new("Test").unwrap();
        let desc = CString::new("A test blueprint").unwrap();
        let domain = CString::new("api").unwrap();
        let result = unsafe {
            agentic_forge_create_blueprint(name.as_ptr(), desc.as_ptr(), domain.as_ptr())
        };
        assert!(!result.is_null());
        unsafe { agentic_forge_free_string(result) };
    }

    #[test]
    fn test_free_null_string() {
        unsafe { agentic_forge_free_string(std::ptr::null_mut()) };
    }
}
