#[macro_use]
extern crate nes;

#[macro_use]
extern crate gfx;

extern crate object_pool;
extern crate gfx_window_glutin as gfx_glutin;
extern crate gfx_device_gl as gfx_gl;
extern crate cgmath;
extern crate glutin;
extern crate image;

pub mod types;

#[macro_use]
pub mod macros;

pub mod storage;
pub use storage::Storage;

pub mod render;
use render::Render;

pub mod process;
pub use process::Process;

//use process::Process;

pub fn main() {
    let (render_join_handler, render_sender)=Render::run();
    let process_join_handler=Process::run(render_sender);

    render_join_handler.join();
    process_join_handler.join();
}