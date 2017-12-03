use gfx;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx_gl;

use gfx::traits::FactoryExt;

use render::Error;

pub type TerrainPSO=gfx::PipelineState<gfx_gl::Resources, TerrainPipeline::Meta>;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex TerrainVertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline TerrainPipeline {
        vbuf: gfx::VertexBuffer<TerrainVertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

pub fn create_terrain_pso(gfx_factory: &mut gfx_gl::Factory) -> Result<TerrainPSO,Error> {
    let rasterizer = gfx::state::Rasterizer::new_fill();
    let primitive = gfx::Primitive::TriangleList;

    let shader=try!(gfx_factory.link_program(
        include_bytes!("../../../shader/triangle_150.glslv"),
        include_bytes!("../../../shader/triangle_150.glslf"),
    ), Error::CompileShaderError);

    let pso=match gfx_factory.create_pipeline_from_program( &shader, primitive, rasterizer, TerrainPipeline::new() ) {
        Ok(pso) => pso,
        Err(error) => return err!(Error::CreatePSOError, Box::new(format!("{}",error))),
    };

    ok!(pso)
}