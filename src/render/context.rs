use gfx;
use gfx_gl;
use gfx_glutin;
use glutin;
//use pipeline::pipe;
use ::pipe;

use gfx::traits::FactoryExt;
//use glutin::GlContext;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub struct Context {
    pub window: glutin::GlWindow,
    pub events_loop:glutin::EventsLoop,

    //clear_color: [f32; 4],
    pub device: gfx_gl::Device,
    pub encoder: gfx::Encoder<gfx_gl::Resources, gfx_gl::CommandBuffer>,
    pub pso: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    //pso_wire: gfx::PipelineState<gfx_gl::Resources, pipe::Meta>,
    pub factory: gfx_gl::Factory,
    //font: rusttype::Font<'static>,
    pub data: pipe::Data<gfx_gl::Resources>,
    //start_time: time::Instant,
}

impl Context {
    pub fn new(events_loop:&) -> Self {
        let mut events_loop = glutin::EventsLoop::new();
        let window_config = glutin::WindowBuilder::new()
            .with_title("Triangle example".to_string())
            .with_dimensions(1024, 768);
        let context = glutin::ContextBuilder::new()
            .with_vsync(true);

        let (window, mut device, mut factory, main_color, mut main_depth) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop);
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
        let pso = factory.create_pipeline_simple(
            include_bytes!("../../shader/triangle_150.glslv"),
            include_bytes!("../../shader/triangle_150.glslf"),
            pipe::new()
        ).unwrap();
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&::TRIANGLE, ());
        let mut data = pipe::Data {
            vbuf: vertex_buffer,
            out: main_color
        };

        Context{
            window,
            events_loop,

            device,
            encoder,
            pso,
            factory,
            data
        }
    }
}