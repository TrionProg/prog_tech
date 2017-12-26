use gfx;
use gfx_gl;

use types::*;

use gfx::Factory;
use gfx::traits::FactoryExt;

use super::ObjectVertex;
use super::TraceVertex;

use render::Error;

pub trait Lod:Sized {
    type V;

    fn new(buffer:Vec<Self::V>, gfx_factory: &mut gfx_gl::Factory) -> Result<Self,Error>;
}

pub struct ObjectLod {
    pub vertex_buffer:gfx::handle::Buffer<gfx_gl::Resources, ObjectVertex>,
    pub slice:gfx::Slice<gfx_gl::Resources>
}

impl Lod for ObjectLod {
    type V=ObjectVertex;

    fn new(buffer:Vec<Self::V>, gfx_factory: &mut gfx_gl::Factory) -> Result<Self,Error> {
        let (vertex_buffer, slice) = gfx_factory.create_vertex_buffer_with_slice(&buffer[..], ());

        let lod=ObjectLod {
            vertex_buffer,
            slice
        };

        ok!(lod)
    }
}



pub struct TraceLod {
    pub vertex_buffer:gfx::handle::Buffer<gfx_gl::Resources, TraceVertex>,
    pub slice:gfx::Slice<gfx_gl::Resources>
}

impl Lod for TraceLod {
    type V=TraceVertex;

    fn new(buffer:Vec<Self::V>, gfx_factory: &mut gfx_gl::Factory) -> Result<Self,Error> {
        let (vertex_buffer, slice) = gfx_factory.create_vertex_buffer_with_slice(&buffer[..], ());

        let lod=TraceLod {
            vertex_buffer,
            slice
        };

        ok!(lod)
    }
}