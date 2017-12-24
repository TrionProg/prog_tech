use nes::{ErrorInfo,ErrorInfoTrait};
use glutin;
use gfx_glutin;

use glutin::GlContext;

use super::Error;
use super::Targets;

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

    pub fn resize(&mut self, width:u32, height:u32, targets:&mut Targets) {
        self.window.resize(width,height);
        gfx_glutin::update_views(&self.window, &mut targets.final_color, &mut targets.final_depth);

        self.width=width;
        self.height=height;
    }
}