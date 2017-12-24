use nes::{ErrorInfo,ErrorInfoTrait};
use glutin;
use gfx_glutin;

use glutin::GlContext;

use super::Error;
use super::{RenderTarget, DepthStencil};

/*
pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;
color_target: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
depth_target: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
pub type RenderTarget = gfx::handle::RenderTargetView<gfx_gl::Resources, ColorFormat>;
pub type DepthStencil = gfx::handle::DepthStencilView<gfx_gl::Resources, DepthFormat>;
*/

pub struct Window {
    window: glutin::GlWindow,
    width:u32,
    height:u32
}

impl Window {
    pub fn new(window:glutin::GlWindow, width:u32, height:u32) -> Self {
        Window {
            window,
            width,
            height
        }
    }

    pub fn swap_buffers(&mut self) -> Result<(),Error> {
        try!(self.window.swap_buffers(), Error::SwapBuffersError);
        ok!()
    }

    pub fn resize(&mut self, width:u32, height:u32, render_target:&mut RenderTarget, depth_stencil:&mut DepthStencil) {
        self.window.resize(width,height);
        gfx_glutin::update_views(&self.window, render_target, depth_stencil);

        self.width=width;
        self.height=height;
    }
}