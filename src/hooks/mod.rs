use detour::GenericDetour;
use log::log;
use std::sync::Mutex;
use crate::cef;

pub static mut URL_REQUEST_CRATE_HOOK: Option<GenericDetour<unsafe extern "system" fn(*const cef::CefRequest) -> *const cef::CefString>> = None;
pub static mut RESOURCE_HANDLER_HOOK: Option<GenericDetour<unsafe extern "system" fn(*const cef::CefRequest) -> *const cef::CefString>> = None;

pub fn initialize_hook() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Hook CefURLRequest_Create
        let url_request_function = cef::get_original_function("CefURLRequest_Create")?;
        let url_request_detour = GenericDetour::new(
            url_request_function, 
            hook_url_request_create
        )?;
        url_request_detour.enable()?;
        URL_REQUEST_CRATE_HOOK = Some(url_request_detour);

        // Hook CefResourceHandler
        let resource_handler_function = cef::get_original_function("CefResourceHandler::ProcessRequest")?;
        let resource_handler_detour = GenericDetour::new(
            resource_handler_function, 
            hook_resource_handler
        )?;
        resource_handler_detour.enable()?;
        RESOURCE_HANDLER_HOOK = Some(resource_handler_detour);
    }
    
    Ok(())
}

pub fn cleanup_hooks() {
    unsafe {
        if let Some(ref hook) = URL_REQUEST_CRATE_HOOK {
            let _ = hook.disable();
        }
        if let Some(ref hook) = RESOURCE_HANDLER_HOOK {
            let _ = hook.disable();
        }
    }
}

unsafe extern "system" fn hook_url_request_create(request: *const cef::CefRequest) -> *const cef::CefString {
    log::debug!("spook: intercepted URL request creation");

    // Get the original function pointer and call it
    unsafe  {
        if let Some(ref hook) = URL_REQUEST_CRATE_HOOK {
            let result = hook.call(request);

        log::debug!("spook: URL request result: {:?}", result);
        result
        } else {
            0
        }
    }
}