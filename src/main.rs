extern crate nalgebra_glm as glm;
use std::env;

use log::{error, info};
use env_logger::Env;

mod app;

fn main() {


    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug");

    env_logger::init_from_env(env);
    
    match env::current_exe() {
        Ok(exe_path) => info!("ex path: \"{}\"", exe_path.display()),
        Err(e) => error!("failed to get current exe path: {e}"),
    }

    match app::main(app::ApplicationConfig {}) {
        Ok(_) => info!("app closed gracefully"),
        Err(e) => error!("app ended in error: {:?}", e),
    }
}
