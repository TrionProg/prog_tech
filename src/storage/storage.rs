use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use object_pool;

use types::*;

use std::marker::PhantomData;

use std::ops::DerefMut;
use std::sync::{Arc,Mutex};
use self::object_pool::growable::Pool;

use render::StorageSender;
use render::{StorageCommand, LoadTexture, LoadMesh, LoadLod};

use render::storage::ObjectVertex;
use render::storage::ObjectMesh;

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
    storage_sender:StorageSender,

    textures_rgb:InnerTextureStorage<RgbTextureID>,
    //textures_rgba:InnerTextureStorage<RgbTextureID>,

    object_meshes:InnerMeshStorage<ObjectMeshID>,

    object_lods:InnerLodStorage<ObjectLodID>
}

impl InnerStorage {
    fn new(storage_sender:StorageSender) -> Self {
        InnerStorage {
            storage_sender,

            textures_rgb:InnerTextureStorage::new(),

            object_meshes:InnerMeshStorage::new(),

            object_lods:InnerLodStorage::new()
        }
    }
}

impl Storage {
    pub fn new(storage_sender:StorageSender) -> Self {
        let inner=InnerStorage::new(storage_sender);

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

impl TextureStorage<RgbTextureID, RgbImage> for Storage {
    fn load_texture(&self, image_buffer:RgbImage) -> Result<RgbTextureID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let texture_id=storage.textures_rgb.insert();

        channel_send!(storage.storage_sender, LoadTexture::RGB(image_buffer, texture_id.clone()).into());

        ok!(texture_id)
    }

    fn delete_texture(&self, texture_id:RgbTextureID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

impl MeshStorage<ObjectMeshID, ObjectMesh> for Storage {
    fn load_mesh(&self, mesh:ObjectMesh) -> Result<ObjectMeshID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let mesh_id=storage.object_meshes.insert();

        channel_send!(storage.storage_sender, LoadMesh::Object(mesh, mesh_id.clone()).into());

        ok!(mesh_id)
    }

    fn delete_mesh(&self, mesh_id:ObjectMeshID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

impl LodStorage<ObjectLodID, ObjectVertex> for Storage {
    fn load_lod(&self, vertex_buffer:Vec<ObjectVertex>) -> Result<ObjectLodID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let lod_id=storage.object_lods.insert();

        channel_send!(storage.storage_sender, LoadLod::Object(vertex_buffer, lod_id.clone()).into());

        ok!(lod_id)
    }

    fn delete_lod(&self, lod_id:ObjectLodID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

/*
impl TextureStorage<RgbaTextureID, RgbaImage> for Storage {
    fn load_texture(&self, image_buffer:RgbaImage) -> Result<RgbaTextureID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let id=storage.textures_rgba.insert(());
        let texture_id=RgbaTextureID::new(id);

        ok!(texture_id)
    }

    fn delete_texture(&self, texture_id:RgbaTextureID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}
*/


