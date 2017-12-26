
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use gfx_gl;

use types::*;
use cgmath::Rad;

use gfx::Factory;
use gfx::traits::FactoryExt;

use cgmath::Matrix4;
use cgmath::Vector3;

use render;
use render::Targets;
use render::Error;
use render::Encoder;

use super::Storage;

pub trait Mesh {
}

pub struct ObjectMesh {
    pub lod:ObjectLodID,
    pub texture:RgbaTextureID,
}

impl ObjectMesh {
    pub fn new(lod:ObjectLodID, texture:RgbaTextureID) -> Self {
        ObjectMesh {
            lod,
            texture
        }
    }

    pub fn draw(&self, storage:&Storage, encoder:&mut Encoder, targets:&Targets,
        x:u32, y:f32, z:u32
    ) -> Result<(),Error> {
        let lod_id=self.lod;
        let lod=storage.object_lods.get(lod_id)?;
        let texture=storage.textures_rgba.get(self.texture)?;

        let model_matrix=Matrix4::from_translation(Vector3::new(x as f32,y, z as f32));

        let data = render::pipelines::ObjectPipeline::Data {
            globals: storage.object_globals.clone(),
            model_matrix: model_matrix.into(),
            texture: (texture.view.clone(), storage.object_pso.sampler.clone()),
            vbuf: lod.vertex_buffer.clone(),

            color_target: targets.final_color.clone(),
            depth_target: targets.final_depth.clone()
        };

        encoder.draw(&lod.slice, &storage.object_pso.pso, &data);

        ok!()
    }
}

impl Mesh for ObjectMesh{}



pub struct TerrainMesh {
    pub lod:ObjectLodID,
}

impl TerrainMesh {
    pub fn new(lod:ObjectLodID) -> Self {
        TerrainMesh {
            lod
        }
    }

    pub fn draw(&self, storage:&Storage, encoder:&mut Encoder, targets:&Targets,
        x:u32, z:u32, texture_id:RgbaTextureID
    ) -> Result<(),Error> {
        let lod_id=self.lod;
        let lod=storage.object_lods.get(lod_id)?;
        let texture=storage.textures_rgba.get(texture_id)?;

        let model_matrix=Matrix4::from_translation(Vector3::new(x as f32,0.0, z as f32));

        let data = render::pipelines::ObjectPipeline::Data {
            globals: storage.object_globals.clone(),
            model_matrix: model_matrix.into(),
            texture: (texture.view.clone(), storage.object_pso.sampler.clone()),
            vbuf: lod.vertex_buffer.clone(),

            color_target: targets.final_color.clone(),
            depth_target: targets.final_depth.clone()
        };

        encoder.draw(&lod.slice, &storage.object_pso.pso, &data);

        ok!()
    }
}

impl Mesh for TerrainMesh{}


pub struct TraceMesh {
    pub lod:TraceLodID,
}

impl TraceMesh {
    pub fn new(lod:TraceLodID) -> Self {
        TraceMesh {
            lod
        }
    }

    pub fn draw(&self, storage:&Storage, encoder:&mut Encoder, targets:&Targets,
                x:u32, z:u32, angle:f32, color:[f32;4]
    ) -> Result<(),Error> {
        let lod_id=self.lod;
        let lod=storage.trace_lods.get(lod_id)?;

        let pos_matrix=Matrix4::from_translation(Vector3::new(x as f32,0.6, z as f32));
        let rot_matrix=Matrix4::from_angle_y(Rad(angle));

        let model_matrix=pos_matrix*rot_matrix;

        let data = render::pipelines::TracePipeline::Data {
            globals: storage.trace_globals.clone(),
            model_matrix: model_matrix.into(),
            color: color,
            vbuf: lod.vertex_buffer.clone(),

            color_target: targets.final_color.clone(),
            depth_target: targets.final_depth.clone()
        };

        encoder.draw(&lod.slice, &storage.trace_pso.pso, &data);

        ok!()
    }
}

impl Mesh for TraceMesh{}