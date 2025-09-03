use std::sync::Once;
use std::fmt::Result;
use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, };


static INIT: Once = Once::new();

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _: *mut ())
    -> bool
{
    match call_reason {
        DLL_PROCESS_ATTACH => {
            INIT.call_once(|| {
                if let Err(e) = initialize() {
                    eprintln!("Failed to initialize: {:?}", e);
                }
            })
        },
        DLL_PROCESS_DETACH => {
            INIT.call_once(|| {
                if let Err(e) = cleanup() {
                    eprintln!("Failed to cleanup: {:?}", e);
                }
            })
        },
        _ => ()
    }

    true
}

fn initialize() -> Result {
    Ok(())
}

fn cleanup() -> Result {
    Ok(())
}