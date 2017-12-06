use nes::{ErrorInfo,ErrorInfoTrait};
use glutin;

use glutin::GlContext;

use super::Error;
use super::{RenderTarget, DepthStencil};

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
        self.width=width;
        self.height=height;
    }
}