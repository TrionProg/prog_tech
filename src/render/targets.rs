use gfx;
use gfx_gl;

pub type FinalColorFormat = gfx::format::Rgba8;
pub type FinalDepthFormat = gfx::format::DepthStencil;
pub type FinalColorTargetView = gfx::handle::RenderTargetView<gfx_gl::Resources, FinalColorFormat>;
pub type FinalDepthTargetView = gfx::handle::DepthStencilView<gfx_gl::Resources, FinalDepthFormat>;
pub type FinalColorTarget = gfx::BlendTarget<FinalColorFormat>;
pub type FinalDepthTarget = gfx::DepthTarget<FinalDepthFormat>;

pub struct Targets {
    pub final_color:FinalColorTargetView,
    pub final_depth:FinalDepthTargetView,
}