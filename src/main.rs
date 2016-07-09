extern crate x11;
extern crate libc;

#[macro_use]
extern crate lazy_static;

mod screen;
mod display;

mod window_manager;
use window_manager::WindowManager;

use std::error::Error;

fn main() {
    let mut wm = match WindowManager::new() {
        Some(wm) => wm,
        None => {
            println!("brwm: Cannot create window manager");
            return;
        }
    };

    match wm.run() {
        Ok(_) => {
            println!("Success!");
            return;
        },
        Err(e) => {
            println!("Error: {}", e.description());
            return;
        }
    }
}
