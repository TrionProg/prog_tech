use gfx;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx_gl;
use render;

use types::*;

use std::marker::PhantomData;

use object_pool::growable::Pool;

use gfx::traits::FactoryExt;
use gfx_gl::Factory;

use storage::{TextureID, MeshID, LodID};

use render::Error;
use render::pipelines::{ObjectPSO, create_object_pso};
use render::pipelines::{TracePSO, create_trace_pso};

use super::ObjectVertex;
use super::TraceVertex;

use super::texture::*;
use super::mesh::*;
use super::lod::*;

pub trait TextureStorage<ID:TextureID,IB> {
    fn load_texture(&mut self, image_buffer:IB, texture_id:ID) -> Result<(), Error>;
    fn delete_texture(&mut self, texture_id:ID) -> Result<(), Error>;
}

pub trait MeshStorage<ID:MeshID,M:Mesh> {
    fn load_mesh(&mut self, mesh:M, mesh_id:ID) -> Result<(), Error>;
    fn delete_mesh(&mut self, mesh_id:ID) -> Result<(), Error>;
}

pub trait LodStorage<ID:LodID,L:Lod,V> {
    fn load_lod(&mut self, vertex_buffer:Vec<V>, lod_id:ID) -> Result<(), Error>;
    fn delete_lod(&mut self, lod_id:ID) -> Result<(), Error>;
}

pub struct Storage {
    pub gfx_factory: Factory,
    pub object_pso: ObjectPSO,
    pub trace_pso: TracePSO,
    //pub fake_texture = u32;

    pub textures_rgba:InnerTextureStorage<RgbaTextureID, RgbaImage, RgbaTexture>,

    pub object_meshes:InnerMeshStorage<ObjectMeshID,ObjectMesh>,
    pub terrain_meshes:InnerMeshStorage<TerrainMeshID,TerrainMesh>,
    pub trace_meshes:InnerMeshStorage<TraceMeshID,TraceMesh>,

    pub object_lods:InnerLodStorage<ObjectLodID,ObjectVertex,ObjectLod>,
    pub trace_lods:InnerLodStorage<TraceLodID,TraceVertex,TraceLod>,

    pub object_globals: gfx::handle::Buffer<gfx_gl::Resources, render::pipelines::object::ObjectGlobals>,
    pub trace_globals: gfx::handle::Buffer<gfx_gl::Resources, render::pipelines::trace::TraceGlobals>,
}

impl Storage {
    pub fn new(mut gfx_factory: Factory) -> Result<Self,Error> {
        let object_pso=create_object_pso(&mut gfx_factory)?;
        let trace_pso=create_trace_pso(&mut gfx_factory)?;
        //let fake_texture = load_texture_raw(&mut gfx_factory, Size2{w: 2, h: 2}, &[0; 4]);

        let storage=Storage {
            gfx_factory:gfx_factory.clone(),
            object_pso,
            trace_pso,
            //fake_texture

            textures_rgba:InnerTextureStorage::new(&gfx_factory),

            object_meshes:InnerMeshStorage::new(),
            terrain_meshes:InnerMeshStorage::new(),
            trace_meshes:InnerMeshStorage::new(),

            object_lods:InnerLodStorage::new(&gfx_factory),
            trace_lods:InnerLodStorage::new(&gfx_factory),


            object_globals:gfx_factory.create_constant_buffer(1),
            trace_globals:gfx_factory.create_constant_buffer(1),
        };

        ok!(storage)
    }
}

pub struct InnerTextureStorage<ID:TextureID,IB,T:Texture<IB=IB>> {
    gfx_factory: Factory,
    pool:Pool<T, T>,
    _phantom_data:PhantomData<(ID,IB)>
}

impl<ID:TextureID,IB,T:Texture<IB=IB>> InnerTextureStorage<ID,IB,T> {
    fn new(gfx_factory: &Factory) -> Self {
        InnerTextureStorage {
            gfx_factory:gfx_factory.clone(),
            pool:Pool::new(),
            _phantom_data:PhantomData
        }
    }

    fn load(&mut self, image_buffer:IB, texture_id:ID) -> Result<(), Error> {
        let texture=T::new(image_buffer, &mut self.gfx_factory)?;

        let id=self.pool.insert(texture);
        assert_eq!(id, texture_id.get_id());

        ok!()
    }

    pub fn get(&self, texture_id:ID) -> Result<&T, Error> {
        match self.pool.get(texture_id.get_id()) {
            Some(texture) => ok!(texture),
            None => err!(Error::NoTexture)
        }
    }

    fn delete(&mut self, texture_id:ID) -> Result<(), Error> {
        ok!()
    }
}

pub struct InnerMeshStorage<ID:MeshID,M:Mesh> {
    pool:Pool<M, M>,
    _phantom_data:PhantomData<(ID)>
}

impl<ID:MeshID,M:Mesh> InnerMeshStorage<ID,M> {
    fn new() -> Self {
        InnerMeshStorage {
            pool:Pool::new(),
            _phantom_data:PhantomData
        }
    }

    fn load(&mut self, mesh:M, mesh_id:ID) -> Result<(), Error> {
        let id=self.pool.insert(mesh);
        assert_eq!(id, mesh_id.get_id());

        ok!()
    }

    pub fn get(&self, mesh_id:ID) -> Result<&M, Error> {
        match self.pool.get(mesh_id.get_id()) {
            Some(mesh) => ok!(mesh),
            None => err!(Error::NoMesh)
        }
    }

    fn delete(&mut self, mesh_id:ID) -> Result<(), Error> {
        ok!()
    }
}

pub struct InnerLodStorage<ID:LodID,V,L:Lod<V=V>> {
    gfx_factory: Factory,
    pool:Pool<L, L>,
    _phantom_data:PhantomData<(ID,V)>
}

impl<ID:LodID,V,L:Lod<V=V>> InnerLodStorage<ID,V,L> {
    fn new(gfx_factory: &Factory) -> Self {
        InnerLodStorage {
            gfx_factory:gfx_factory.clone(),
            pool:Pool::new(),
            _phantom_data:PhantomData
        }
    }

    fn load(&mut self, vertex_buffer:Vec<V>, lod_id:ID) -> Result<(), Error> {
        let lod=L::new(vertex_buffer, &mut self.gfx_factory)?;
        
        let id=self.pool.insert(lod);
        assert_eq!(id, lod_id.get_id());

        ok!()
    }

    pub fn get(&self, lod_id:ID) -> Result<&L, Error> {
        match self.pool.get(lod_id.get_id()) {
            Some(lod) => ok!(lod),
            None => err!(Error::NoLod)
        }
    }

    fn delete(&mut self, lod_id:ID) -> Result<(), Error> {
        ok!()
    }
}


impl TextureStorage<RgbaTextureID, RgbaImage> for Storage {
    fn load_texture(&mut self, image_buffer:RgbaImage, texture_id:RgbaTextureID) -> Result<(), Error> {
        self.textures_rgba.load(image_buffer, texture_id)
    }

    fn delete_texture(&mut self, texture_id:RgbaTextureID) -> Result<(), Error> {
        self.textures_rgba.delete(texture_id)
    }
}

impl MeshStorage<ObjectMeshID, ObjectMesh> for Storage {
    fn load_mesh(&mut self, mesh:ObjectMesh, mesh_id:ObjectMeshID) -> Result<(),Error> {
        self.object_meshes.load(mesh, mesh_id)
    }

    fn delete_mesh(&mut self, mesh_id:ObjectMeshID) -> Result<(),Error> {
        self.object_meshes.delete(mesh_id)
    }
}

impl MeshStorage<TerrainMeshID, TerrainMesh> for Storage {
    fn load_mesh(&mut self, mesh:TerrainMesh, mesh_id:TerrainMeshID) -> Result<(),Error> {
        self.terrain_meshes.load(mesh, mesh_id)
    }

    fn delete_mesh(&mut self, mesh_id:TerrainMeshID) -> Result<(),Error> {
        self.terrain_meshes.delete(mesh_id)
    }
}

impl MeshStorage<TraceMeshID, TraceMesh> for Storage {
    fn load_mesh(&mut self, mesh:TraceMesh, mesh_id:TraceMeshID) -> Result<(),Error> {
        self.trace_meshes.load(mesh, mesh_id)
    }

    fn delete_mesh(&mut self, mesh_id:TraceMeshID) -> Result<(),Error> {
        self.trace_meshes.delete(mesh_id)
    }
}

impl LodStorage<ObjectLodID, ObjectLod, ObjectVertex> for Storage {
    fn load_lod(&mut self, vertex_buffer:Vec<ObjectVertex>, lod_id:ObjectLodID) -> Result<(),Error> {
        self.object_lods.load(vertex_buffer, lod_id)
    }

    fn delete_lod(&mut self, lod_id:ObjectLodID) -> Result<(),Error> {
        self.object_lods.delete(lod_id)
    }
}

impl LodStorage<TraceLodID, TraceLod, TraceVertex> for Storage {
    fn load_lod(&mut self, vertex_buffer:Vec<TraceVertex>, lod_id:TraceLodID) -> Result<(),Error> {
        self.trace_lods.load(vertex_buffer, lod_id)
    }

    fn delete_lod(&mut self, lod_id:TraceLodID) -> Result<(),Error> {
        self.trace_lods.delete(lod_id)
    }
}

