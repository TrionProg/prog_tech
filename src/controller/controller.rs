use std;
use glutin;
use nes::{ErrorInfo,ErrorInfoTrait};

use types::*;

use std::thread;
use std::thread::JoinHandle;

use process;
use process::ProcessSender;
use process::ProcessCommand;

use process;
use process::RenderSender;
use process::RenderCommand;

use super::Error;
use super::ControllerCommand;

pub type ControllerSender = std::sync::mpsc::Sender<ControllerCommand>;
pub type ControllerReceiver = std::sync::mpsc::Receiver<ControllerCommand>;

pub struct Controller {
    events_loop:glutin::EventsLoop,
}

impl Controller{
    pub fn run()-> (JoinHandle<()>, ControllerSender) {
        let (controller_sender, controller_receiver) = std::sync::mpsc::channel();

        let join_handle=thread::Builder::new().name("Controller".to_string()).spawn(move|| {
            let process_sender = match controller_receiver.recv() {
                Ok( ControllerCommand::ProcessSender(process_sender) ) => process_sender,
                _ => recv_error!(ControllerCommand::ProcessSender),
            };

            let render_sender = match controller_receiver.recv() {
                Ok( ControllerCommand::RenderSender(render_sender) ) => render_sender,
                _ => recv_error!(ControllerCommand::RenderSender),
            };

            let render_sender = match controller_receiver.recv() {
                Ok( ControllerCommand::RenderSender(render_sender) ) => render_sender,
                _ => recv_error!(ControllerCommand::RenderSender),
            };

            let mut controller=match Self::setup(controller_receiver, render_sender.clone(), process_sender.clone()) {
                Ok(controller) => controller,
                Err(error) => {
                    println!("Controller setup error: {}", error);

                    try_send![process_sender, ProcessCommand::ControllerSetupError];
                    try_send![render_sender, RenderCommand::ControllerSetupError];

                    return;
                }
            };

            controller.synchronize_setup();

            match render.lifecycle() {
                Ok(_) => {
                    //do something

                    render.synchronize_finish();
                }
                Err(error) => {
                    println!("Controller Error: {}!", error);

                    match error {
                        Error::ProcessThreadCrash(_,source) => {
                            /*
                            if source==ThreadSource::Disk {
                                try_send![disk.storage_sender, StorageCommand::IpcListenerThreadCrash(source)];
                            }
                            */
                        }
                        _ => {
                            try_send![render.process_sender, ProcessCommand::ControllerThreadCrash(ThreadSource::Controller)];
                        }
                    }
                }
            }
        }).unwrap();

        (join_handle, controller_sender, storage_sender)
    }

    fn setup(controller_receiver:ControllerReceiver, storage_receiver:StorageReceiver, process_sender:ProcessSender) -> Result<Self,Error> {
        let mut events_loop = glutin::EventsLoop::new();

        let window_config = glutin::WindowBuilder::new()
            .with_title("Triangle example".to_string())
            .with_dimensions(1024, 768);
        let context = glutin::ContextBuilder::new()
            .with_vsync(true);

        let (gfx_window, gfx_device,mut gfx_factory, controller_target, depth_stencil) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop);

        let window=Window::new(gfx_window, 1024, 768);
        let mut encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();
        let mut storage=Storage::new(gfx_factory)?;

        let scheduler=Scheduler::new(50);

        let render=Controller {
            controller_receiver,
            storage_receiver,
            process_sender,

            window,
            events_loop,
            controller_target,
            depth_stencil,

            gfx_device,
            encoder,
            storage,
            scheduler
        };

        ok!(render)
    }

    fn synchronize_setup(&mut self) {
        try_send![self.process_sender, ProcessCommand::ControllerIsReady];

        match self.controller_receiver.recv() {
            Ok( ControllerCommand::ProcessIsReady ) => {},
            _ => recv_error!(ControllerCommand::ProcessIsReady),
        }
    }

    fn lifecycle(&mut self) -> Result<(),Error> {
        self.lifecycle_render()
    }

    fn lifecycle_render(&mut self) -> Result<(),Error> {
        loop {
            self.scheduler.frame_begin=Time::now();

            self.poll_window_events()?;

            if self.handle_controller_commands()? {
                return ok!();
            }

            self.handle_storage_commands()?;
            //animate
            self.render()?;

            self.scheduler.frame_end=Time::now();

            match self.scheduler.make_plan() {
                Some(wait) => thread::sleep(wait),
                None => {},
            }
        }
    }

    fn poll_window_events(&mut self) -> Result<(),Error> {
        let mut quit=false;

        let window=&mut self.window;
        let controller_target=&mut self.controller_target;
        let depth_stencil=&mut self.depth_stencil;

        self.events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                            .. },
                        ..
                    } | glutin::WindowEvent::Closed => quit=true,
                    glutin::WindowEvent::Resized(width, height) =>
                        window.resize(width, height, controller_target, depth_stencil),
                    _ => {},
                }
            }
        });

        if quit {
            channel_send!(self.process_sender, ProcessCommand::Quit);
        }

        self.scheduler.poll_window_events_end=Time::now();

        ok!()
    }

    fn handle_controller_commands(&mut self) -> Result<bool,Error> {
        loop {
            let command = match self.controller_receiver.try_recv() {
                Ok(command) => command,
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) =>
                    return err!(Error::ControllerThreadCrash, ThreadSource::Controller)
            };

            match command {
                ControllerCommand::ProcessThreadCrash(source) => return err!(Error::ProcessThreadCrash, source),

                ControllerCommand::Shutdown =>
                    return ok!(true),
                _ => {},
            }
        }

        self.scheduler.handle_controller_commands_end=Time::now();

        ok!(false)
    }

    fn handle_storage_commands(&mut self) -> Result<(),Error> {
        let until=self.scheduler.handle_controller_commands_end+self.scheduler.plan_storage_commands_handling_i;

        loop {
            loop {
                let command = match self.storage_receiver.try_recv() {
                    Ok(command) => command,
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) =>
                        return err!(Error::ControllerThreadCrash, ThreadSource::Controller)
                };

                match command {
                    StorageCommand::LoadTexture(load_texture) =>
                        self.load_texture(load_texture)?,
                    StorageCommand::LoadMesh(load_mesh) =>
                        self.load_mesh(load_mesh)?,
                    StorageCommand::LoadLod(load_lod) =>
                        self.load_lod(load_lod)?,
                }
            }

            let now=Time::now();

            if Time::now() < until {
                thread::sleep(Duration::new(0,1000_000));
            }else{
                self.scheduler.handle_storage_commands_end=now;
                break;
            }
        }

        ok!()
    }

    fn render(&mut self) -> Result<(),Error> {
        self.encoder.clear(&self.controller_target, CLEAR_COLOR);

        match self.ren() {
            Ok(_) => {},
            Err(e) => println!("{}",e)
        }

        //self.encoder.draw(&slice, &self.storage.terrain_pso, &data);
        self.encoder.flush(&mut self.gfx_device);

        self.window.swap_buffers()?;
        self.gfx_device.cleanup();

        self.scheduler.rendering_end=Time::now();

        ok!()
    }

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
            out: self.controller_target.clone(),
            out_depth: self.depth_stencil.clone()
        };

        self.encoder.draw(&lod.slice, &self.storage.object_pso, &data);

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

    fn synchronize_finish(&mut self) {
        try_send![self.process_sender, ProcessCommand::ControllerFinished];

        match self.controller_receiver.recv() {
            Ok( ControllerCommand::ProcessFinished ) => {},
            _ => recv_error!(ControllerCommand::ProcessFinished),
        }
    }

    /*
    fn renderol(&mut self) -> Result<(),Error> {
        //use ::pipe;
        use gfx::traits::FactoryExt;
        let (vertex_buffer, slice) = self.storage.gfx_factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
        /*
        let mut data = super::pipelines::TerrainPipeline::Data {
            vbuf: vertex_buffer,
            out: self.controller_target.clone()
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
            out: self.controller_target.clone(),
            out_depth: self.depth_stencil.clone()
        };

        let mut next_frame=Time::now();

        loop {
            loop {
                let command = match self.controller_receiver.try_recv() {
                    Ok(command) => command,
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) =>
                        return err!(Error::ControllerThreadCrash, ThreadSource::Controller)
                };

                match command {
                    ControllerCommand::LoadTexture(load_texture) =>
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
            self.encoder.clear(&self.controller_target, CLEAR_COLOR);
            self.encoder.draw(&slice, &self.storage.terrain_pso, &data);
            self.encoder.flush(&mut self.gfx_device);

            self.window.swap_buffers()?;
            self.gfx_device.cleanup();
        }

        ok!()
    }
    */
}