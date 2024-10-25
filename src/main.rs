extern crate nalgebra_glm as glm;

mod app;

fn main() {
    match app::main(app::ApplicationConfig {}) {
        Ok(_) => println!("app closed gracefully"),
        Err(e) => println!("app ended in error: {:?}", e),
    }
}
