use gfx;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx_gl;

use types::*;

use std::marker::PhantomData;

use object_pool::growable::Pool;

//use gfx::traits::FactoryExt;

use storage::{TextureID, MeshID, LodID};

use render::Error;
use render::pipelines::{ObjectPSO, create_object_pso};

use super::ObjectVertex;

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
    pub gfx_factory: gfx_gl::Factory,
    pub object_pso: ObjectPSO,
    //pub fake_texture = u32;

    pub textures_rgb:InnerTextureStorage<RgbTextureID, RgbImage, RgbTexture>,

    pub object_meshes:InnerMeshStorage<ObjectMeshID,ObjectMesh>,

    pub object_lods:InnerLodStorage<ObjectLodID,ObjectVertex,ObjectLod>
}

impl Storage {
    pub fn new(mut gfx_factory: gfx_gl::Factory) -> Result<Self,Error> {
        let object_pso=create_object_pso(&mut gfx_factory)?;
        //let fake_texture = load_texture_raw(&mut gfx_factory, Size2{w: 2, h: 2}, &[0; 4]);

        let storage=Storage {
            gfx_factory:gfx_factory.clone(),
            object_pso,
            //fake_texture
            textures_rgb:InnerTextureStorage::new(&gfx_factory),

            object_meshes:InnerMeshStorage::new(),

            object_lods:InnerLodStorage::new(&gfx_factory),
        };

        ok!(storage)
    }
}

pub struct InnerTextureStorage<ID:TextureID,IB,T:Texture<IB=IB>> {
    gfx_factory: gfx_gl::Factory,
    pool:Pool<T, T>,
    _phantom_data:PhantomData<(ID,IB)>
}

impl<ID:TextureID,IB,T:Texture<IB=IB>> InnerTextureStorage<ID,IB,T> {
    fn new(gfx_factory: &gfx_gl::Factory) -> Self {
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
    gfx_factory: gfx_gl::Factory,
    pool:Pool<L, L>,
    _phantom_data:PhantomData<(ID,V)>
}

impl<ID:LodID,V,L:Lod<V=V>> InnerLodStorage<ID,V,L> {
    fn new(gfx_factory: &gfx_gl::Factory) -> Self {
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


impl TextureStorage<RgbTextureID, RgbImage> for Storage {
    fn load_texture(&mut self, image_buffer:RgbImage, texture_id:RgbTextureID) -> Result<(), Error> {
        self.textures_rgb.load(image_buffer, texture_id)
    }

    fn delete_texture(&mut self, texture_id:RgbTextureID) -> Result<(), Error> {
        self.textures_rgb.delete(texture_id)
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

impl LodStorage<ObjectLodID, ObjectLod, ObjectVertex> for Storage {
    fn load_lod(&mut self, vertex_buffer:Vec<ObjectVertex>, lod_id:ObjectLodID) -> Result<(),Error> {
        self.object_lods.load(vertex_buffer, lod_id)
    }

    fn delete_lod(&mut self, lod_id:ObjectLodID) -> Result<(),Error> {
        self.object_lods.delete(lod_id)
    }
}