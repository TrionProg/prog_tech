use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use object_pool;

use types::*;

use std::marker::PhantomData;

use std::ops::DerefMut;
use std::sync::{Arc,Mutex};
use self::object_pool::growable::Pool;

use render::RenderSender;
use render::{RenderCommand, LoadTexture, LoadMesh, LoadLod};

use render::storage::{ObjectVertex,TraceVertex};
use render::storage::{ObjectMesh,TerrainMesh,TraceMesh};

use super::Error;
use super::{TextureID, MeshID, LodID};


pub trait TextureStorage<ID:TextureID,IB> {
    fn load_texture(&self, image_buffer:IB) -> Result<ID, Error>;
    fn delete_texture(&self, texture_id:ID) -> Result<(), Error>;
}

pub trait MeshStorage<ID:MeshID,M> {
    fn load_mesh(&self, mesh:M) -> Result<ID, Error>;
    fn delete_mesh(&self, mesh_id:ID) -> Result<(), Error>;
}

pub trait LodStorage<ID:LodID,V> {
    fn load_lod(&self, vertex_buffer:Vec<V>) -> Result<ID, Error>;
    fn delete_lod(&self, lod_id:ID) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct Storage {
    inner:Arc<Mutex<InnerStorage>>,
}

struct InnerStorage {
    render_sender:RenderSender,

    textures_rgba:InnerTextureStorage<RgbaTextureID>,
    //textures_rgbaa:InnerTextureStorage<RgbaTextureID>,

    object_meshes:InnerMeshStorage<ObjectMeshID>,
    terrain_meshes:InnerMeshStorage<TerrainMeshID>,
    trace_meshes:InnerMeshStorage<TraceMeshID>,

    object_lods:InnerLodStorage<ObjectLodID>,
    trace_lods:InnerLodStorage<TraceLodID>
}

impl InnerStorage {
    fn new(render_sender:RenderSender) -> Self {
        InnerStorage {
            render_sender,

            textures_rgba:InnerTextureStorage::new(),

            object_meshes:InnerMeshStorage::new(),
            terrain_meshes:InnerMeshStorage::new(),
            trace_meshes:InnerMeshStorage::new(),

            object_lods:InnerLodStorage::new(),
            trace_lods:InnerLodStorage::new()
        }
    }
}

impl Storage {
    pub fn new(render_sender:RenderSender) -> Self {
        let inner=InnerStorage::new(render_sender);

        Storage {
            inner:Arc::new(Mutex::new(inner))
        }
    }
}

struct InnerTextureStorage<ID:TextureID> {
    pool:Pool<(),()>,
    _phantom_data:PhantomData<ID>
}

impl<ID:TextureID> InnerTextureStorage<ID> {
    fn new() -> Self {
        InnerTextureStorage {
            pool:Pool::new(),
            _phantom_data:PhantomData
        }
    }

    fn insert(&mut self) -> ID {
        let id=self.pool.insert(());
        let texture_id=ID::new(id);

        texture_id
    }
}

struct InnerMeshStorage<ID:MeshID> {
    pool:Pool<(),()>,
    _phantom_data:PhantomData<ID>
}

impl<ID:MeshID> InnerMeshStorage<ID> {
    fn new() -> Self {
        InnerMeshStorage {
            pool:Pool::new(),
            _phantom_data:PhantomData
        }
    }

    fn insert(&mut self) -> ID {
        let id=self.pool.insert(());
        let mesh_id=ID::new(id);

        mesh_id
    }
}

struct InnerLodStorage<ID:LodID> {
    pool:Pool<(),()>,
    _phantom_data:PhantomData<ID>
}

impl<ID:LodID> InnerLodStorage<ID> {
    fn new() -> Self {
        InnerLodStorage {
            pool:Pool::new(),
            _phantom_data:PhantomData
        }
    }

    fn insert(&mut self) -> ID {
        let id=self.pool.insert(());
        let lod_id=ID::new(id);

        lod_id
    }
}

impl TextureStorage<RgbaTextureID, RgbaImage> for Storage {
    fn load_texture(&self, image_buffer:RgbaImage) -> Result<RgbaTextureID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let texture_id=storage.textures_rgba.insert();

        try_send!(storage.render_sender, LoadTexture::RGBA(image_buffer, texture_id.clone()).into());

        ok!(texture_id)
    }

    fn delete_texture(&self, texture_id:RgbaTextureID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

impl MeshStorage<ObjectMeshID, ObjectMesh> for Storage {
    fn load_mesh(&self, mesh:ObjectMesh) -> Result<ObjectMeshID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let mesh_id=storage.object_meshes.insert();

        try_send!(storage.render_sender, LoadMesh::Object(mesh, mesh_id.clone()).into());

        ok!(mesh_id)
    }

    fn delete_mesh(&self, mesh_id:ObjectMeshID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

impl MeshStorage<TerrainMeshID, TerrainMesh> for Storage {
    fn load_mesh(&self, mesh:TerrainMesh) -> Result<TerrainMeshID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let mesh_id=storage.terrain_meshes.insert();

        try_send!(storage.render_sender, LoadMesh::Terrain(mesh, mesh_id.clone()).into());

        ok!(mesh_id)
    }

    fn delete_mesh(&self, mesh_id:TerrainMeshID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

impl MeshStorage<TraceMeshID, TraceMesh> for Storage {
    fn load_mesh(&self, mesh:TraceMesh) -> Result<TraceMeshID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let mesh_id=storage.trace_meshes.insert();

        try_send!(storage.render_sender, LoadMesh::Trace(mesh, mesh_id.clone()).into());

        ok!(mesh_id)
    }

    fn delete_mesh(&self, mesh_id:TraceMeshID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

impl LodStorage<ObjectLodID, ObjectVertex> for Storage {
    fn load_lod(&self, vertex_buffer:Vec<ObjectVertex>) -> Result<ObjectLodID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let lod_id=storage.object_lods.insert();

        try_send!(storage.render_sender, LoadLod::Object(vertex_buffer, lod_id.clone()).into());

        ok!(lod_id)
    }

    fn delete_lod(&self, lod_id:ObjectLodID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

impl LodStorage<TraceLodID, TraceVertex> for Storage {
    fn load_lod(&self, vertex_buffer:Vec<TraceVertex>) -> Result<TraceLodID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let lod_id=storage.trace_lods.insert();

        try_send!(storage.render_sender, LoadLod::Trace(vertex_buffer, lod_id.clone()).into());

        ok!(lod_id)
    }

    fn delete_lod(&self, lod_id:TraceLodID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

/*
impl TextureStorage<RgbaaTextureID, RgbaaImage> for Storage {
    fn load_texture(&self, image_buffer:RgbaaImage) -> Result<RgbaaTextureID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let id=storage.textures_rgbaa.insert(());
        let texture_id=RgbaaTextureID::new(id);

        ok!(texture_id)
    }

    fn delete_texture(&self, texture_id:RgbaaTextureID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}
*/


