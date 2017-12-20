use gfx;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx_gl;

use gfx::traits::FactoryExt;
use gfx::Factory;

use render::Error;

//pub type ObjectPSO=gfx::PipelineState<gfx_gl::Resources, ObjectPipeline::Meta>;

pub struct ObjectPSO {
    pub pso:gfx::PipelineState<gfx_gl::Resources, ObjectPipeline::Meta>,
    pub sampler:gfx::handle::Sampler<gfx_gl::Resources>,
}

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    /*
    vertex ObjectVertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline ObjectPipeline {
        vbuf: gfx::VertexBuffer<ObjectVertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
    */
    vertex ObjectVertex {
        pos: [f32; 3] = "a_pos",
        uv: [f32; 2] = "a_uv",
    }

    pipeline ObjectPipeline {
        basic_color: gfx::Global<[f32; 4]> = "u_basic_color",
        final_matrix: gfx::Global<[[f32; 4]; 4]> = "u_final_matrix",
        vbuf: gfx::VertexBuffer<ObjectVertex> = (),
        texture: gfx::TextureSampler<[f32; 4]> = "t_tex",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl ObjectVertex {
    pub fn new(pos:[i32;3], uv:[i32;2]) -> Self {
        ObjectVertex {
            pos: [pos[0] as f32, pos[2] as f32, pos[1] as f32],
            uv: [uv[0] as f32, uv[1] as f32]
        }
    }
}

pub fn create_object_pso(gfx_factory: &mut gfx_gl::Factory) -> Result<ObjectPSO,Error> {
    let rasterizer = gfx::state::Rasterizer::new_fill();
    let primitive = gfx::Primitive::TriangleList;

    let shader=try!(gfx_factory.link_program(
        include_bytes!("shaders/v.glsl"),
        include_bytes!("shaders/f.glsl"),
    ), Error::CompileShaderError);

    let pso=match gfx_factory.create_pipeline_from_program( &shader, primitive, rasterizer, ObjectPipeline::new() ) {
        Ok(pso) => pso,
        Err(error) => return err!(Error::CreatePSOError, Box::new(format!("{}",error))),
    };

    let sampler_info=gfx::texture::SamplerInfo::new(gfx::texture::FilterMethod::Bilinear, gfx::texture::WrapMode::Tile);
    let sampler = gfx_factory.create_sampler(sampler_info);

    let object_pso=ObjectPSO{
        pso,
        sampler
    };

    ok!(object_pso)
}