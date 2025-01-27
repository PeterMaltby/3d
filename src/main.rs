use clap::Parser;
use config::AppConfig;
use env_logger::Env;
use winit::event_loop::EventLoop;

use crate::prelude::*;
use app::App;
use error::Result;

mod app;
mod config;
mod error;
pub mod prelude;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    //custom config file path
    #[arg(short, long)]
    config: Option<String>,
    //logging override
    #[arg(short, long)]
    log: Option<String>,
    // fullscreen override
    #[arg(short, long)]
    fullscreen: Option<bool>,
}

fn main() -> Result<()> {
    // parse command line args
    let args = Args::parse();

    // load app config
    let mut app_config = AppConfig::new(args.config)?;

    // command line configuration overrides
    match args.fullscreen {
        Some(bool) => app_config.fullscreen = Some(bool),
        _ => (),
    }
    match args.log {
        Some(rust_log_string) => app_config.rust_log = Some(rust_log_string),
        _ => (),
    }

    // init logger
    match &app_config.rust_log {
        Some(rust_log_string) => {
            let env = Env::default().filter_or("RUST_LOG", rust_log_string);
            env_logger::init_from_env(env);
            info!("logging enabled!;\n {}", rust_log_string);
        }
        _ => env_logger::init(),
    }

    // create event loop
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    // Create our app using the app configuration
    let mut app = App::new(app_config);

    info!("starting app");
    event_loop.run_app(&mut app)?;

    // print exit status and terminate
    match app.exit_state {
        Ok(_) => {
            info!("app exited with succesful status");
            return Ok(());
        }
        Err(e) => {
            error!("app exited in error: {}", e);
            return Err(e.into());
        }
    }
}

