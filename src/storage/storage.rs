use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use object_pool;

use types::*;

use std::ops::DerefMut;
use std::sync::{Arc,Mutex};
use self::object_pool::growable::Pool;

use render::RenderSender;
use render::RenderCommand;

use super::Error;
use super::TextureID;


pub trait TextureStorage<ID:TextureID,IB> {
    fn load_texture(&self, texture_buffer:IB) -> Result<ID, Error>;
    fn delete_texture(&self, texture_id:ID) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct Storage {
    inner:Arc<Mutex<InnerStorage>>,
}

struct InnerStorage {
    render_sender:RenderSender,

    textures_rgb:Pool<(),()>,
    textures_rgba:Pool<(),()>,
}

impl InnerStorage {
    fn new(render_sender:RenderSender) -> Self {
        InnerStorage {
            render_sender,

            textures_rgb:Pool::new(),
            textures_rgba:Pool::new()
        }
    }
}

impl Storage {
    fn new(render_sender:RenderSender) -> Self {
        let inner=InnerStorage::new(render_sender);

        Storage {
            inner:Arc::new(Mutex::new(inner))
        }
    }
}

impl TextureStorage<RgbTextureID, RgbImage> for Storage {
    fn load_texture(&self, texture_buffer:RgbImage) -> Result<RgbTextureID, Error> {
        mutex_lock!(&self.inner => storage, Error);

        let id=storage.textures_rgb.insert(());
        let texture_id=RgbTextureID::new(id);

        ok!(texture_id)
    }

    fn delete_texture(&self, texture_id:RgbTextureID) -> Result<(), Error> {
        mutex_lock!(&self.inner => storage, Error);

        ok!()
    }
}

/*
impl TextureStorage<RgbaTextureID, RgbaImage> for Storage {
    fn load_texture(&self, texture_buffer:RgbaImage) -> Result<RgbaTextureID, Error> {
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


