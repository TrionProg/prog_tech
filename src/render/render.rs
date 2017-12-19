use std;
use gfx;
use gfx_gl;
use gfx_glutin;
use glutin;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;

use gfx::Device;
use glutin::GlContext;
use glutin::EventsLoop;

use std::thread;
use std::thread::JoinHandle;

/*
use process;
use process::SupervisorSender;
use process::SupervisorCommand;
*/

use supervisor;
use supervisor::SupervisorSender;
use supervisor::SupervisorCommand;

use controller;
use controller::ControllerSender;
use controller::ControllerCommand;

use super::Error;
use super::Window;
use super::Storage;
use super::RenderCommand;
use super::{LoadTexture, LoadMesh, LoadLod};

pub type RenderSender = reactor::Sender<ThreadSource,RenderCommand>;
pub type RenderReceiver = reactor::Receiver<ThreadSource,RenderCommand>;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub type RenderTarget = gfx::handle::RenderTargetView<gfx_gl::Resources, ColorFormat>;
pub type DepthStencil = gfx::handle::DepthStencilView<gfx_gl::Resources, DepthFormat>;

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];


pub struct Render {
    render_receiver:RenderReceiver,
    supervisor_sender:SupervisorSender,
    controller_sender:ControllerSender,

    window: Window,
    render_target:RenderTarget,
    depth_stencil:DepthStencil,


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
        let (render_sender, mut render_receiver) = reactor::create_channel(ThreadSource::Render);

        let join_handle=thread::Builder::new().name("Render".to_string()).spawn(move|| {
            let (mut supervisor_sender, mut controller_sender) = Self::get_senders(&mut render_receiver).unwrap();

            println!("R1");

            let mut events_loop = glutin::EventsLoop::new();

            let mut render=match Self::setup(render_receiver, supervisor_sender.clone(), controller_sender.clone(), &events_loop) {
                Ok(render) => render,
                Err(error) => {
                    println!("Render setup error: {}", error);

                    send![
                        supervisor_sender, SupervisorCommand::ThreadCrash(ThreadSource::Render),
                        controller_sender, ControllerCommand::ThreadCrash(ThreadSource::Render)
                    ].unwrap();

                    return;
                }
            };

            println!("R2");


            send![
                 controller_sender, ControllerCommand::EventsLoop(events_loop)
            ].unwrap();

            render.synchronize_setup().unwrap();

            println!("R3");

            match render.lifecycle() {
                Ok(_) => {
                    //do something

                    println!("R4");

                    render.synchronize_finish().unwrap();
                }
                Err(error) => {
                    println!("Render Error: {}!", error);

                    match error {
                        Error::ThreadCrash(_,thread) => {
                            /*
                            if source==ThreadSource::Disk {
                                try_send![disk.storage_sender, StorageCommand::IpcListenerThreadCrash(source)];
                            }
                            */
                        }
                        _ => {
                            send![
                                supervisor_sender , SupervisorCommand::ThreadCrash(ThreadSource::Render),
                                controller_sender , ControllerCommand::ThreadCrash(ThreadSource::Render)
                            ].unwrap();
                        }
                    }
                }
            }

            println!("R5");
        }).unwrap();

        (join_handle, render_sender)
    }

    fn get_senders(receiver:&mut RenderReceiver) -> Result<(SupervisorSender, ControllerSender),Error> {
        let supervisor_sender=wait![receiver,
            RenderCommand::SupervisorSender(supervisor_sender) => supervisor_sender
        ].unwrap();

        let controller_sender=wait![receiver,
            RenderCommand::ControllerSender(controller_sender) => controller_sender
        ].unwrap();

        ok!((supervisor_sender, controller_sender))
    }

    fn setup(render_receiver:RenderReceiver, supervisor_sender:SupervisorSender, controller_sender:ControllerSender, events_loop:&EventsLoop) -> Result<Self,Error> {
        let window_config = glutin::WindowBuilder::new()
            .with_title("Triangle example".to_string())
            .with_dimensions(1024, 768);
        let context = glutin::ContextBuilder::new()
            .with_vsync(true);

        let (gfx_window, gfx_device,mut gfx_factory, render_target, depth_stencil) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(window_config, context, events_loop);

        let window=Window::new(gfx_window, 1024, 768);
        let mut encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();
        let mut storage=Storage::new(gfx_factory)?;

        let render=Render {
            render_receiver,
            supervisor_sender,
            controller_sender,

            window,
            render_target,
            depth_stencil,

            gfx_device,
            encoder,
            storage
        };

        ok!(render)
    }

    fn synchronize_setup(&mut self) -> Result<(),Error>{
        try_send![self.supervisor_sender, SupervisorCommand::ThreadReady(ThreadSource::Render)];

        wait![self.render_receiver,
            RenderCommand::SupervisorReady => ()
        ].unwrap();

        ok!()
    }

    fn lifecycle(&mut self) -> Result<(),Error> {
        loop {
            self.render()?;

            if self.handle_render_commands()? {
                println!("QUIT4");
                return ok!();
            }
        }
    }

    fn handle_render_commands(&mut self) -> Result<bool,Error> {
        loop {
            match try_recv_block!(self.render_receiver) {
                RenderCommand::ThreadCrash(thread) => return err!(Error::ThreadCrash, thread),
                RenderCommand::Tick => return ok!(false),
                RenderCommand::Shutdown => return ok!(true),

                RenderCommand::ResizeWindow(width, height) =>
                    self.resize_window(width,height)?,

                RenderCommand::LoadTexture(load_texture) =>
                    self.load_texture(load_texture)?,
                RenderCommand::LoadMesh(load_mesh) =>
                    self.load_mesh(load_mesh)?,
                RenderCommand::LoadLod(load_lod) =>
                    self.load_lod(load_lod)?,
                _ => unreachable!()
            }
        }
    }

    fn render(&mut self) -> Result<(),Error> {
        self.gfx_device.cleanup();
        self.encoder.clear(&self.render_target, CLEAR_COLOR);

        /*
        match self.ren() {
            Ok(_) => {},
            Err(e) => println!("{}",e)
        }
        */

        //self.encoder.draw(&slice, &self.storage.terrain_pso, &data);
        self.encoder.flush(&mut self.gfx_device);

        self.window.swap_buffers()?;

        ok!()
    }
/*
    fn ren(&mut self) -> Result<(),Error> {
        use cgmath::Matrix4;
        use storage::mesh::MeshID;
        use gfx::traits::FactoryExt;
        use object_pool::growable::ID;
        use cgmath::SquareMatrix;

        let (lod_id,texture_id)={
            let mesh=self.storage.object_meshes.get(ObjectMeshID::new(ID::new(0)))?;
            (mesh.lod, mesh.texture)
        };

        let texture=self.storage.textures_rgba.get(texture_id)?;
        let lod=self.storage.object_lods.get(lod_id)?;

        let sampler = self.storage.gfx_factory.create_sampler_linear();

        let data = super::pipelines::ObjectPipeline::Data {
            basic_color: [1.0, 1.0, 1.0, 1.0],
            final_matrix: Matrix4::identity().into(),
            vbuf: lod.vertex_buffer.clone(),
            texture: (texture.view.clone(), sampler),
            out: self.render_target.clone(),
            out_depth: self.depth_stencil.clone()
        };

        self.encoder.draw(&lod.slice, &self.storage.object_pso, &data);

        ok!()
    }

*/

    fn resize_window(&mut self, width:u32, height:u32) -> Result<(),Error> {
        self.window.resize(width, height, &mut self.render_target, &mut self.depth_stencil);

        ok!()
    }

    fn load_texture(&mut self, load_texture:LoadTexture) -> Result<(),Error> {
        use super::storage::TextureStorage;

        match load_texture {
            LoadTexture::RGBA(image_buffer, texture_id) =>
                self.storage.load_texture(image_buffer, texture_id)
        }
    }

    fn load_mesh(&mut self, load_mesh:LoadMesh) -> Result<(),Error> {
        use super::storage::MeshStorage;

        match load_mesh {
            LoadMesh::Object(object_mesh, mesh_id) =>
                self.storage.load_mesh(object_mesh, mesh_id)
        }
    }

    fn load_lod(&mut self, load_lod:LoadLod) -> Result<(),Error> {
        use super::storage::LodStorage;

        match load_lod {
            LoadLod::Object(vertex_buffer, lod_id) =>
                self.storage.load_lod(vertex_buffer, lod_id)
        }
    }

    fn synchronize_finish(&mut self) -> Result<(),Error>{
        println!("R F1");
        try_send![self.supervisor_sender, SupervisorCommand::ThreadFinished(ThreadSource::Render)];

        wait![self.render_receiver,
            RenderCommand::SupervisorFinished => ()
        ].unwrap();

        println!("R F");

        ok!()
    }

    /*
    fn renderol(&mut self) -> Result<(),Error> {
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

        let mut next_frame=Time::now();

        loop {
            loop {
                let command = match self.render_receiver.try_recv() {
                    Ok(command) => command,
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) =>
                        return err!(Error::RenderThreadCrash, ThreadSource::Render)
                };

                match command {
                    RenderCommand::LoadTexture(load_texture) =>
                        self.load_texture(load_texture)?
                }
            }
        }

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
    */
}