#[macro_use]
extern crate nes;

#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin as gfx_glutin;
extern crate gfx_device_gl as gfx_gl;
extern crate cgmath;
extern crate glutin;
extern crate image;

pub mod render;
use render::Render;

pub fn main() {
    let render_join_handler=Render::run();

    render_join_handler.join();
}