#[unsafe(no_mangle)]
pub extern "C" fn mquad_arcanoid_crate_version() -> u32 {
    1
}

#[cfg(target_family = "wasm")]
mod platform {
    use sapp_jsutils::JsObject;

    mod imports {
        use sapp_jsutils::JsObject;

        unsafe extern "C" {
            pub fn app_done_loading();
            pub fn app_is_on_mobile() -> bool;
            pub fn app_get_orientation() -> f32;
            pub fn panic_screen(msg: JsObject);
        }
    }

    pub fn done_loading() {
        unsafe { imports::app_done_loading() }
    }

    pub fn on_mobile() -> bool {
        unsafe { imports::app_is_on_mobile() }
    }

    pub fn get_orientation() -> f32 {
        unsafe { imports::app_get_orientation() }
    }

    pub fn panic_screen(msg: &str) {
        unsafe {
            imports::panic_screen(JsObject::string(msg));
        }
    }
}

#[cfg(not(target_family = "wasm"))]
mod platform {
    pub fn done_loading() { /* Nothing */
    }

    pub fn on_mobile() -> bool {
        false
    }

    pub fn get_orientation() -> f32 {
        0.0
    }

    pub fn panic_screen(_msg: &str) {}
}

pub use platform::*;
