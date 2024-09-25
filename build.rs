extern crate gl_generator;

use cfg_aliases::cfg_aliases;
use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    cfg_aliases! {
        // Native displays.
        x11_platform: { all(feature = "x11", free_unix, not(wasm_platform)) },
        wayland_platform: { all(feature = "wayland", free_unix, not(wasm_platform)) },

        // Backends.
        egl_backend: { all(feature = "egl", any(windows, unix), not(apple), not(wasm_platform)) },
        glx_backend: { all(feature = "glx", x11_platform, not(wasm_platform)) },
        wgl_backend: { all(feature = "wgl", windows, not(wasm_platform)) },
        cgl_backend: { all(macos_platform, not(wasm_platform)) },
    }

    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");

    let mut file = File::create(dest.join("gl_bindings.rs")).unwrap();
    Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, []).write_bindings(GlobalGenerator, &mut file).unwrap();
}
