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

use super::Error;
use super::Window;
use super::Storage;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

use super::pipelines::TerrainVertex;
const TRIANGLE: [TerrainVertex; 3] = [
    TerrainVertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
    TerrainVertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
    TerrainVertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] }
];

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];


pub struct Render {
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
    pub fn run()-> JoinHandle<()> {
        //let (render_sender, render_receiver) = std::sync::mpsc::channel();

        let join_handle=thread::Builder::new().name("Render".to_string()).spawn(move|| {
            let mut render=match Self::setup() {
                Ok(render) => render,
                Err(e) => {panic!("aaa");}
            };

            match render.lifecycle() {
                Ok(_) => {},
                Err(e) => {
                    use std::io::Write;
                    writeln!(&mut std::io::stderr(), "Render Error: {}!", e);
                }
            }
        }).unwrap();

        join_handle
    }

    fn setup() -> Result<Self,Error> {
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
        let mut data = super::pipelines::TerrainPipeline::Data {
            vbuf: vertex_buffer,
            out: self.render_target.clone()
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