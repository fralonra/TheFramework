use theframework::*;

pub mod demo;

use crate::demo::Demo;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    std::env::set_var("RUST_BACKTRACE", "1");

    let demo = Demo::new();
    let mut app = TheApp::new();

    _ = app.run(Box::new(demo));
}