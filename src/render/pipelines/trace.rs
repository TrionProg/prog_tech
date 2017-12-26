use gfx;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx_gl;

use gfx::traits::FactoryExt;
use gfx::Factory;

use render::Error;

//pub type TracePSO=gfx::PipelineState<gfx_gl::Resources, TracePipeline::Meta>;

pub struct TracePSO {
    pub pso:gfx::PipelineState<gfx_gl::Resources, TracePipeline::Meta>
}

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

//view_matrix: [[f32; 4]; 4] = "u_view_matrix",
//proj_matrix: [[f32; 4]; 4] = "u_proj_matrix",

gfx_defines!{
    constant TraceGlobals {
        proj_view_matrix: [[f32; 4]; 4] = "u_proj_view_matrix",
    }

    vertex TraceVertex {
        pos: [f32; 3] = "a_pos",
    }

    pipeline TracePipeline {
        globals: gfx::ConstantBuffer<TraceGlobals> = "c_globals",
        model_matrix: gfx::Global<[[f32; 4]; 4]> = "u_model_matrix",
        color: gfx::Global<[f32; 4]> = "u_color",
        vbuf: gfx::VertexBuffer<TraceVertex> = (),

        color_target: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        depth_target: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl TraceVertex {
    pub fn new(x:f32,y:f32,z:f32) -> Self {
        TraceVertex {pos:[x,y,z]}
    }
}

pub fn create_trace_pso(gfx_factory: &mut gfx_gl::Factory) -> Result<TracePSO,Error> {
    let rasterizer = gfx::state::Rasterizer::new_fill();
    let primitive = gfx::Primitive::TriangleList;

    let shader=try!(gfx_factory.link_program(
        include_bytes!("shaders/trace_v.glsl"),
        include_bytes!("shaders/trace_f.glsl"),
    ), Error::CompileShaderError);

    let pso=match gfx_factory.create_pipeline_from_program( &shader, primitive, rasterizer, TracePipeline::new() ) {
        Ok(pso) => pso,
        Err(error) => return err!(Error::CreatePSOError, Box::new(format!("{}",error))),
    };

    let trace_pso=TracePSO{
        pso
    };

    ok!(trace_pso)
}