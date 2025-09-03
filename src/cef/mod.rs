use std::ptr::null_mut;
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{LoadLibraryA, GetProcAddress};

pub static mut  ORIGINAL_CEF: *mut std::ffi::c_void = null_mut();

// CEF Request structure
#[repr(C)]
pub struct CefRequest {
    pub vtable: *const CefRequestVTable,
}

#[repr(C)]
pub struct CefRequestVTable {
    pub get_url: extern "system" fn(*const CefRequest) -> *const CefString,
    pub get_method: unsafe extern "system" fn(*const CefRequest) -> *const CefString,
}

// CEF String structure
#[repr(C)]
pub struct CefString {
    pub str: *const u16,
    pub length: usize,
}

pub fn load_original_cef() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let dll_name = std::ffi::CString::new("libcef_original.dll")?;
        let handle = LoadLibraryA(PCSTR(dll_name.as_ptr() as *const u8));
        match handle {
            Ok(hmodule) => {
                ORIGINAL_CEF = hmodule.0 as *mut std::ffi::c_void;
            }
            Err(e) => {
                eprintln!("Failed to load CEF library: {}", e);
                ORIGINAL_CEF = null_mut();
            }
        }
    }

    if unsafe { ORIGINAL_CEF.is_null() } {
        Err("Failed to load CEF library".into())
    } else {
        Ok(())
    }
}

pub fn get_original_function(name: &str) -> Result<*const std::ffi::c_void, Box<dyn std::error::Error>> {
    unsafe {
        let func_name = std::ffi::CString::new(name)?;
        let hmodule = windows::Win32::Foundation::HMODULE(ORIGINAL_CEF as *mut std::ffi::c_void);
        let func_ptr = GetProcAddress(hmodule, PCSTR(func_name.as_ptr() as *const u8));

        if let Some(ptr) = func_ptr {
            Ok(ptr as *const std::ffi::c_void)
        } else {
            Err(format!("Function {} not found in original CEF library", name).into())
        }
    }
}

impl CefString {
    pub fn to_string(&self) -> String {
        if self.str.is_null() || self.length == 0 {
            return String::new();
        }
        let slice = unsafe { std::slice::from_raw_parts(self.str, self.length) };
        String::from_utf16_lossy(slice)
    }
}