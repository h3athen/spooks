use std::sync::Once;
use std::fmt::Result;
use std::fs::File;
use windows::{
    core::PCSTR, Win32::{Foundation::*, System::{Diagnostics::Debug::OutputDebugStringA, SystemServices::*}}
};
use simplelog::{self, WriteLogger, LevelFilter, Config};
use log;

mod cef;
mod hooks;

static INIT: Once = Once::new();

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _: *mut ())
    -> bool
{
    if call_reason == DLL_PROCESS_DETACH {
        let _ = cleanup();
    }
    true
}

fn initialize() -> Result {
    // File in %TEMP% so we can see output when loaded by rundll32 (no console)
    let mut log_path = std::env::temp_dir();
    log_path.push("spooks.log");

    // Try init logger only once. Ignore error if already set.
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(&log_path).unwrap_or_else(|_| File::create("spooks_fallback.log").unwrap()),
    );

    log::info!("spooks: logging initialized at {:?}", log_path);
    log::info!("spooks: initialize() called");

    cef::load_original_cef()?;
    hook::initialize_hook()?;

    // Also emit to debugger (view via DebugView / OutputDebugString)
    output_debug("spooks: initialized");

    log::info!("spooks: CEF hijack initialized successfully");
    Ok(())
}

fn cleanup() -> Result {
    output_debug("spooks: cleanup()");
    hooks::cleanup_hooks();
    Ok(())
}

fn output_debug(msg: &str) {
    // OutputDebugStringA expects a null-terminated C string.
    if let Ok(cstr) = std::ffi::CString::new(msg) {
        unsafe { OutputDebugStringA(PCSTR(cstr.as_ptr() as *const u8)); }
    }
}

// Exported entry point callable via:
//   rundll32.exe path\to\spooks.dll,Run
// Signature per rundll32 contract: void CALLBACK Fn(HWND, HINSTANCE, LPSTR, int)
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Run(hwnd: HWND, hinst: HINSTANCE, cmd_line: *mut u8, show: i32) {
    INIT.call_once(|| {
        if let Err(e) = initialize() {
            output_debug(&format!("spooks: init failed: {:?}", e));
        }
    });

    log::info!("spooks: Run invoked (hwnd={:?}, hinst={:?}, show={})", hwnd, hinst, show);
    if !cmd_line.is_null() {
        // Best effort is for the display of first bytes of command line
        unsafe {
            let slice = std::slice::from_raw_parts(cmd_line, 64.min(strlen(cmd_line)));
            if let Ok(s) = std::str::from_utf8(slice) { log::info!("spooks: cmd_line='{}'", s); }
        }
    }
}

// Simple strlen for the narrow command line pointer (best effort, stops at NUL or 1KB)
fn strlen(ptr: *const u8) -> usize {
    if ptr.is_null() { return 0; }
    let mut len = 0usize;
    unsafe {
        while len < 1024 { // cap
            if *ptr.add(len) == 0 { break; }
            len += 1;
        }
    }
    len
}