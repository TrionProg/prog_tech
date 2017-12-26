use std;
use gfx;
use gfx_gl;
use gfx_glutin;
use glutin;
use nes::{ErrorInfo,ErrorInfoTrait};
use reactor;

use types::*;
use consts::*;

use gfx::Device;
use glutin::GlContext;
use glutin::EventsLoop;

use std::thread;
use std::thread::JoinHandle;

use supervisor;
use supervisor::SupervisorSender;
use supervisor::SupervisorCommand;

use controller;
use controller::ControllerSender;
use controller::ControllerCommand;

use process;
use process::ProcessSender;
use process::ProcessCommand;

use ::Camera as CommonCamera;

use super::Error;
use super::Window;
use super::Targets;
use super::Storage;
use super::Slots;
use super::RenderCommand;
use super::{LoadTexture, LoadMesh, LoadLod, SetSlot};
use super::{Trace,TracePool};

pub type RenderSender = reactor::Sender<ThreadSource,RenderCommand>;
pub type RenderReceiver = reactor::Receiver<ThreadSource,RenderCommand>;

pub type Encoder = gfx::Encoder<gfx_gl::Resources, gfx_gl::CommandBuffer>;

pub use process::{Map,Tile};

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];


pub struct Render {
    render_receiver:RenderReceiver,
    supervisor_sender:SupervisorSender,
    controller_sender:ControllerSender,
    process_sender:ProcessSender,

    window: Window,
    targets:Targets,


    //clear_color: [f32; 4],
    gfx_device: gfx_gl::Device,
    encoder: Encoder,
    //pub pso: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    //pso_wire: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    storage: Storage,
    slots: Slots,
    //font: rusttype::Font<'static>,
    //pub data: pipe::Data<gfx_gl::Resources>,

    resources_loaded:bool,
    camera:CommonCamera,
    map:Option<Map>,
    cursor_pos:(u32,u32),
    cursor_a:Option<(u32,u32)>,
    cursor_b:Option<(u32,u32)>,
    traces:TracePool,
}

impl Render{
    pub fn run()-> (JoinHandle<()>, RenderSender) {
        let (render_sender, mut render_receiver) = reactor::create_channel(ThreadSource::Render);

        let join_handle=thread::Builder::new().name("Render".to_string()).spawn(move|| {
            let (mut supervisor_sender, mut controller_sender, mut process_sender) = Self::get_senders(&mut render_receiver).unwrap();

            let camera=wait![render_receiver,
                RenderCommand::Camera(camera) => camera
            ].unwrap();

            println!("R1");

            let mut events_loop = glutin::EventsLoop::new();

            let mut render=match Self::setup(
                render_receiver,
                supervisor_sender.clone(),
                controller_sender.clone(),
                process_sender.clone(),
                &events_loop,
                camera
            ) {
                Ok(render) => render,
                Err(error) => {
                    println!("Render setup error: {}", error);

                    send![
                        supervisor_sender, SupervisorCommand::ThreadCrash(ThreadSource::Render),
                        controller_sender, ControllerCommand::ThreadCrash(ThreadSource::Render),
                        process_sender, ProcessCommand::ThreadCrash(ThreadSource::Render)
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
                                controller_sender , ControllerCommand::ThreadCrash(ThreadSource::Render),
                                process_sender , ProcessCommand::ThreadCrash(ThreadSource::Render)
                            ].unwrap();
                        }
                    }
                }
            }

            println!("R5");
        }).unwrap();

        (join_handle, render_sender)
    }

    fn get_senders(receiver:&mut RenderReceiver) -> Result<(SupervisorSender, ControllerSender, ProcessSender),Error> {
        let supervisor_sender=wait![receiver,
            RenderCommand::SupervisorSender(supervisor_sender) => supervisor_sender
        ].unwrap();

        let controller_sender=wait![receiver,
            RenderCommand::ControllerSender(controller_sender) => controller_sender
        ].unwrap();

        let process_sender=wait![receiver,
            RenderCommand::ProcessSender(process_sender) => process_sender
        ].unwrap();

        ok!((supervisor_sender, controller_sender, process_sender))
    }

    fn setup(
        render_receiver:RenderReceiver,
        supervisor_sender:SupervisorSender,
        controller_sender:ControllerSender,
        process_sender:ProcessSender,
        events_loop:&EventsLoop,
        camera:CommonCamera
    ) -> Result<Self,Error> {
        let window_config = glutin::WindowBuilder::new()
            .with_title("ProgrammierungTechnologie".to_string())
            .with_dimensions(1024, 768);
        let context = glutin::ContextBuilder::new()
            .with_vsync(true);

        let (
            gfx_window,
            gfx_device,
            mut gfx_factory,
            final_color_target_view,
            final_depth_target_view
        ) = gfx_glutin::init::<super::targets::FinalColorFormat, super::targets::FinalDepthFormat>(window_config, context, events_loop);

        let window=Window::new(gfx_window, 1024, 768);

        let targets=Targets {
            final_color:final_color_target_view,
            final_depth:final_depth_target_view
        };

        let storage=Storage::new(gfx_factory.clone())?;

        let mut encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();

        let render=Render {
            render_receiver,
            supervisor_sender,
            controller_sender,
            process_sender,

            window,
            targets,

            gfx_device,
            encoder,
            storage,
            slots:Slots::new(),

            resources_loaded:false,
            camera,
            map:None,
            cursor_pos:(0,0),
            cursor_a:None,
            cursor_b:None,
            traces:TracePool::new(),
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
                RenderCommand::SetSlot(set_slot) =>
                    self.slots.set_slot(set_slot),
                RenderCommand::CreateMap =>
                    self.map=Some(Map::new()),
                RenderCommand::LoadTile(x,z,tile) => {
                    match self.map {
                        Some(ref mut map) => map.tiles[x][z]=tile,
                        None => {}
                    }
                },

                RenderCommand::MoveCursor(x,z) =>
                    self.cursor_pos=(x,z),
                RenderCommand::SetCursorA(cursor_a) =>
                    self.cursor_a=cursor_a,
                RenderCommand::SetCursorB(cursor_b) =>
                    self.cursor_b=cursor_b,


                RenderCommand::ResourcesReady => {
                    self.resources_loaded=true;
                    try_send!(self.process_sender, ProcessCommand::ResourcesLoaded);
                },
                RenderCommand::CreateTrace(trace) =>
                    self.traces.insert(trace),
                RenderCommand::DeleteTrace(trace_id) =>
                    self.traces.delete(trace_id),

                _ => unreachable!()
            }
        }
    }

    fn render(&mut self) -> Result<(),Error> {
        self.gfx_device.cleanup();
        self.encoder.clear(&self.targets.final_color, CLEAR_COLOR);
        self.encoder.clear_depth(&self.targets.final_depth, 1.0);

        if self.resources_loaded {
            self.render_map()?;
        }

        //self.encoder.draw(&slice, &self.storage.terrain_pso, &data);
        self.encoder.flush(&mut self.gfx_device);

        self.window.swap_buffers()?;

        ok!()
    }

    fn render_map(&mut self) -> Result<(),Error> {
        //use storage::mesh::MeshID;
        //use gfx::traits::FactoryExt;
        //use gfx::Factory;
        //use object_pool::growable::ID;
        //use cgmath::SquareMatrix;
        //use gfx::texture::SamplerInfo;

        let camera=self.camera.get_render_camera()?.unwrap();
        let proj_view_matrix=camera.perspective_matrix * camera.camera_matrix;

        self.encoder.update_constant_buffer(
            &self.storage.object_globals,
            &super::pipelines::object::ObjectGlobals {
                proj_view_matrix: proj_view_matrix.into()
            },
        );

        self.encoder.update_constant_buffer(
            &self.storage.trace_globals,
            &super::pipelines::trace::TraceGlobals {
                proj_view_matrix: proj_view_matrix.into()
            },
        );

        match self.map {
            Some(ref map) => {
                for z in 0..MAP_SIZE {
                    for x in 0..MAP_SIZE {
                        match map.tiles[x][z] {
                            Tile::Air => {},
                            Tile::Floor(index) => {
                                let mesh_id=self.slots.floor_mesh;
                                let texture_id=self.slots.terrain_textures[index];

                                self.storage.terrain_meshes.get(mesh_id)?.draw(
                                    &self.storage, &mut self.encoder, &self.targets,
                                    x as u32,z as u32,texture_id
                                )?;
                            }
                            Tile::Wall(index) => {
                                let r=if x<MAP_SIZE-1 && map.tiles[x+1][z].is_wall() {0}else{1<<0};
                                let l=if x>0 && map.tiles[x-1][z].is_wall() {0}else{1<<1};
                                let f=if z<MAP_SIZE-1 && map.tiles[x][z+1].is_wall() {0}else{1<<2};
                                let b=if z>0 && map.tiles[x][z-1].is_wall() {0}else{1<<3};

                                let mask=r | l | f | b;

                                let mesh_id=self.slots.wall_meshes[mask];
                                let texture_id=self.slots.terrain_textures[index];

                                self.storage.terrain_meshes.get(mesh_id)?.draw(
                                    &self.storage, &mut self.encoder, &self.targets,
                                    x as u32,z as u32,texture_id
                                )?;
                            },
                            Tile::Hole(index) => {
                                let r=if x<MAP_SIZE-1 && map.tiles[x+1][z].is_hole() {0}else{1<<0};
                                let l=if x>0 && map.tiles[x-1][z].is_hole() {0}else{1<<1};
                                let f=if z<MAP_SIZE-1 && map.tiles[x][z+1].is_hole() {0}else{1<<2};
                                let b=if z>0 && map.tiles[x][z-1].is_hole() {0}else{1<<3};

                                let mask=r | l | f | b;

                                let mesh_id=self.slots.hole_meshes[mask];
                                let texture_id=self.slots.terrain_textures[index];

                                self.storage.terrain_meshes.get(mesh_id)?.draw(
                                    &self.storage, &mut self.encoder, &self.targets,
                                    x as u32,z as u32,texture_id
                                )?;
                            },
                        }
                    }
                }
            },
            None => {},
        }

        match self.cursor_a {
            Some((x,z)) => {
                let mesh_id=self.slots.cursor_a;
                self.storage.object_meshes.get(mesh_id)?.draw(
                    &self.storage, &mut self.encoder, &self.targets,
                    x, 0.05, z,
                )?;
            },
            None => {},
        }

        match self.cursor_b {
            Some((x,z)) => {
                let mesh_id=self.slots.cursor_b;
                self.storage.object_meshes.get(mesh_id)?.draw(
                    &self.storage, &mut self.encoder, &self.targets,
                    x, 0.05, z,
                )?;
            },
            None => {},
        }

        let mesh_id=self.slots.tile;
        self.storage.object_meshes.get(mesh_id)?.draw(
            &self.storage, &mut self.encoder, &self.targets,
            1, 0.05,4
        )?;

        let mesh_id=self.slots.cursor;
        self.storage.object_meshes.get(mesh_id)?.draw(
            &self.storage, &mut self.encoder, &self.targets,
            self.cursor_pos.0, 0.1,self.cursor_pos.1,
        )?;

        self.traces.draw(&self.storage, &mut self.encoder, &self.targets)?;

        ok!()
    }

    fn resize_window(&mut self, width:u32, height:u32) -> Result<(),Error> {
        self.window.resize(width, height, &mut self.targets);

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
            LoadMesh::Object(mesh, mesh_id) =>
                self.storage.load_mesh(mesh, mesh_id),
            LoadMesh::Terrain(mesh, mesh_id) =>
                self.storage.load_mesh(mesh, mesh_id),
            LoadMesh::Trace(mesh, mesh_id) =>
                self.storage.load_mesh(mesh, mesh_id),
        }
    }

    fn load_lod(&mut self, load_lod:LoadLod) -> Result<(),Error> {
        use super::storage::LodStorage;

        match load_lod {
            LoadLod::Object(vertex_buffer, lod_id) =>
                self.storage.load_lod(vertex_buffer, lod_id),
            LoadLod::Trace(vertex_buffer, lod_id) =>
                self.storage.load_lod(vertex_buffer, lod_id),
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
}