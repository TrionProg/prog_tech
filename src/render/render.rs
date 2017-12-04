use std;
use gfx;
use gfx_gl;
use gfx_glutin;
use glutin;
use nes::{ErrorInfo,ErrorInfoTrait};

use gfx::Device;
use glutin::GlContext;

use std::thread;
use std::thread::JoinHandle;

use process;
use process::ProcessSender;
use process::ProcessCommand;

use super::Error;
use super::Window;
use super::Storage;
use super::RenderCommand;

pub type RenderSender = std::sync::mpsc::Sender<RenderCommand>;
pub type RenderReceiver = std::sync::mpsc::Receiver<RenderCommand>;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

use super::pipelines::TerrainVertex;
const TRIANGLE: [TerrainVertex; 3] = [
    TerrainVertex { pos: [ -0.5, -0.5, 0.0 ], uv: [0.0, 1.0] },
    TerrainVertex { pos: [  0.5, -0.5, 0.0 ], uv: [1.0, 1.0] },
    TerrainVertex { pos: [  0.0,  0.5, 0.0 ], uv: [0.5, 0.0] }
];

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];


pub struct Render {
    render_receiver:RenderReceiver,
    process_sender:ProcessSender,

    window: Window,
    events_loop:glutin::EventsLoop,
    render_target:gfx::handle::RenderTargetView<gfx_gl::Resources, ColorFormat>,
    depth_stencil:gfx::handle::DepthStencilView<gfx_gl::Resources, DepthFormat>,


    //clear_color: [f32; 4],
    gfx_device: gfx_gl::Device,
    encoder: gfx::Encoder<gfx_gl::Resources, gfx_gl::CommandBuffer>,
    //pub pso: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    //pso_wire: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    storage: Storage,
    //font: rusttype::Font<'static>,
    //pub data: pipe::Data<gfx_gl::Resources>,
}

impl Render{
    pub fn run()-> (JoinHandle<()>, RenderSender) {
        let (render_sender, render_receiver) = std::sync::mpsc::channel();

        let join_handle=thread::Builder::new().name("Render".to_string()).spawn(move|| {
            let process_sender = match render_receiver.recv() {
                Ok( RenderCommand::ProcessSender(process_sender) ) => process_sender,
                _ => recv_error!(RenderCommand::ProcessSender),
            };

            let mut render=match Self::setup(render_receiver, process_sender.clone()) {
                Ok(render) => render,
                Err(e) => {
                    //error!("Render setup error: {}", error);

                    try_send![process_sender, ProcessCommand::RenderSetupError];

                    return;
                }
            };

            match render.lifecycle() {
                Ok(_) => {},
                Err(e) => {
                    use std::io::Write;
                    writeln!(&mut std::io::stderr(), "Render Error: {}!", e);
                }
            }
        }).unwrap();

        (join_handle, render_sender)
    }

    fn setup(render_receiver:RenderReceiver, process_sender:ProcessSender) -> Result<Self,Error> {
        let mut events_loop = glutin::EventsLoop::new();

        let window_config = glutin::WindowBuilder::new()
            .with_title("Triangle example".to_string())
            .with_dimensions(1024, 768);
        let context = glutin::ContextBuilder::new()
            .with_vsync(true);

        let (gfx_window, gfx_device,mut gfx_factory, render_target, depth_stencil) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop);

        let window=Window::new(gfx_window, 1024, 768);
        let mut encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();
        let mut storage=Storage::new(gfx_factory)?;

        let render=Render {
            render_receiver,
            process_sender,

            window,
            events_loop,
            render_target,
            depth_stencil,

            gfx_device,
            encoder,
            storage
        };

        ok!(render)
    }

    fn lifecycle(&mut self) -> Result<(),Error> {
        //use ::pipe;
        use gfx::traits::FactoryExt;
        let (vertex_buffer, slice) = self.storage.gfx_factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
        /*
        let mut data = super::pipelines::TerrainPipeline::Data {
            vbuf: vertex_buffer,
            out: self.render_target.clone()
        };
        */

        use cgmath::{Vector2, Matrix4, SquareMatrix, Array};

        use gfx::handle::{ShaderResourceView};

        fn load_texture_raw<R, F>(factory: &mut F) -> ShaderResourceView<R, [f32; 4]>
            where R: gfx::Resources, F: gfx::Factory<R>
        {
            use gfx::texture;

            const data: [[u8; 4]; 4] = [
                [ 0xFF, 0x00, 0x00, 0xFF ], [ 0x00, 0xFF, 0x00, 0xFF ],
                [ 0x00, 0x00, 0xFF, 0xFF ], [ 0xFF, 0xFF, 0xFF, 0xFF ],
            ];

            //let kind = texture::Kind::D2(2 as texture::Size, 2 as texture::Size, texture::AaMode::Single);
            let (_, view) = factory.create_texture_immutable::<ColorFormat>(
                texture::Kind::D2(2, 2, texture::AaMode::Single),
                //texture::Mipmap::Provided,
                &[&data]
            ).unwrap();
            //let (_, view) = factory.create_texture_const_u8::<ColorFormat>(kind, &[data]).unwrap();
            view
        }

        fn load_texture<R, F>(factory: &mut F) -> ShaderResourceView<R, [f32; 4]>
            where R: gfx::Resources, F: gfx::Factory<R>
        {
            use std::fs::{File};
            use std::io::{Read};
            use std::io::{Cursor};
            use image;
            use gfx::texture;

            let mut buf = Vec::new();
            let fullpath = "img.png";//&Path::new("img.png");//.join(&path);
            let mut file = match File::open(&fullpath) {
                Ok(file) => file,
                Err(err) => {
                    panic!("Can`t open file '{}' ({})", fullpath, err);
                },
            };
            let cursor=match file.read_to_end(&mut buf) {
                Ok(_) => Cursor::new(buf),
                Err(err) => {
                    panic!("Can`t read file '{}' ({})", fullpath, err);
                },
            };

            let img = image::load(cursor, image::PNG).unwrap().to_rgba();
            let (w, h) = img.dimensions();
            let data=&img.into_vec();

            let (_, view) = factory.create_texture_immutable_u8::<ColorFormat>(
                texture::Kind::D2(w as texture::Size, h as texture::Size, texture::AaMode::Single),
                //texture::Mipmap::Provided,
                &[&data[..]]
            ).unwrap();
            //let (_, view) = factory.create_texture_const_u8::<ColorFormat>(kind, &[data]).unwrap();
            view
        }

        let sampler = self.storage.gfx_factory.create_sampler_linear();
        let fake_texture = load_texture_raw(&mut self.storage.gfx_factory);
        let tex=load_texture(&mut self.storage.gfx_factory);

        let data = super::pipelines::TerrainPipeline::Data {
            basic_color: [1.0, 1.0, 1.0, 1.0],
            final_matrix: Matrix4::identity().into(),
            vbuf: vertex_buffer,
            texture: (tex, sampler),
            out: self.render_target.clone(),
            out_depth: self.depth_stencil.clone()
        };

        let mut running = true;
        while running {
            // fetch events
            self.events_loop.poll_events(|event| {
                if let glutin::Event::WindowEvent { event, .. } = event {
                    match event {
                        glutin::WindowEvent::KeyboardInput {
                            input: glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                                .. },
                            ..
                        } | glutin::WindowEvent::Closed => running = false,
                        glutin::WindowEvent::Resized(width, height) => {
                            //self.window.resize(width, height);
                            //gfx_glutin::update_views(&context.window, &mut context.data.out, &mut context.main_depth);
                        },
                        _ => (),
                    }
                }
            });

            // draw a frame
            self.encoder.clear(&self.render_target, CLEAR_COLOR);
            self.encoder.draw(&slice, &self.storage.terrain_pso, &data);
            self.encoder.flush(&mut self.gfx_device);

            self.window.swap_buffers()?;
            self.gfx_device.cleanup();
        }

        ok!()
    }
}