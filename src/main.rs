#[macro_use]
extern crate nes;

#[macro_use]
extern crate reactor;

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

pub mod supervisor;
pub use supervisor::Supervisor;

pub mod render;

pub mod process;

pub mod controller;

pub mod location;

pub mod camera;
pub use camera::Camera;

pub fn main() {
    Supervisor::run();
}