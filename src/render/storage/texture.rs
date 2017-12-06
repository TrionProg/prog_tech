use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use gfx;
use gfx_gl;

use types::*;

use gfx::Factory;

use gfx::texture::Size;
use gfx::texture::Kind;
use gfx::texture::AaMode;

use render::Error;

pub trait Texture:Sized {
    type IB;

    fn new(image_buffer:Self::IB, gfx_factory: &mut gfx_gl::Factory) -> Result<Self,Error>;
}

pub struct RgbTexture {
    texture:gfx::handle::Texture<gfx_gl::Resources, gfx::format::R8_G8_B8_A8>,
    pub view:gfx::handle::ShaderResourceView<gfx_gl::Resources, [f32; 4]>
}

impl Texture for RgbTexture {
    type IB=RgbImage;

    fn new(image_buffer:Self::IB, gfx_factory: &mut gfx_gl::Factory) -> Result<Self,Error> {
        let width=image_buffer.width() as Size;
        let height=image_buffer.height() as Size;

        let data=image_buffer.into_vec();

        let (texture, view) = try!( gfx_factory.create_texture_immutable_u8::<gfx::format::Rgba8>(
            Kind::D2(width, height, AaMode::Single),
            &[&data[..]]
        ), Error::CreateTextureError);

        let texture=RgbTexture {
            texture,
            view
        };

        ok!(texture)
    }
}