extern crate nalgebra_glm as glm;
use std::env;

mod app;

fn main() {
    
    match env::current_exe() {
        Ok(exe_path) => println!("ex path: \"{}\"", exe_path.display()),
        Err(e) => println!("failed to get current exe path: {e}"),
    }

    match app::main(app::ApplicationConfig {}) {
        Ok(_) => println!("app closed gracefully"),
        Err(e) => println!("app ended in error: {:?}", e),
    }
}
