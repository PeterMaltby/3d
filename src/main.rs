
mod app;

fn main() {
    match app::main(app::AppConfig {}) {
        Ok(_) => println!("app closed gracefully"),
        Err(e) => println!("app ended in error: {:?}", e),
    }
}
