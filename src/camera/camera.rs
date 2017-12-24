use std;
use nes::{ErrorInfo,ErrorInfoTrait};
use cgmath;
use render;
use location::*;

use std::sync::{Arc,Mutex};
use std::ops::DerefMut;

use cgmath::{Vector2,Vector3,PerspectiveFov,Basis3,Rotation3};
use cgmath::{vec2,vec3,Rad};
use glutin::ElementState;
use glutin::MouseScrollDelta;

use controller::Input;

use super::Viewport;
use super::Error;

#[derive(Clone)]
pub struct Camera{
    inner:Arc<Mutex<InnerCamera>>
}

struct InnerCamera {
    center_position: Pos3D,
    angle: Vector2<f32>,
    distance:f32,
    camera_matrix:Matrix4,
    camera_position:Pos3D,

    viewport:Option<Viewport>,
}

impl InnerCamera{
    fn new(window_width:u32, window_height:u32) -> Self {
        use cgmath::SquareMatrix;

        let mut camera=InnerCamera{
            center_position: Pos3D::new(9.0,0.0,9.0),
            angle: vec2(-3.14/4.0,3.14/4.0),
            distance: 10.0,
            camera_matrix:Matrix4::identity(),
            camera_position:Pos3D::new(0.0,0.0,0.0),

            viewport:Viewport::configure(window_width, window_height),
        };

        camera.calc_matrix();

        camera
    }

    fn rotate(&mut self, input:&Input){
        let mouse_move_x=match self.viewport {
            Some( ref viewport ) => input.mouse_move_x as f32 / viewport.width as f32,
            None => 0.0,
        };

        let mouse_move_y=match self.viewport {
            Some( ref viewport ) => input.mouse_move_y as f32 / viewport.height as f32,
            None => 0.0,
        };

        self.angle.y-=mouse_move_x*3.14*1.5;
        self.angle.x-=mouse_move_y*3.14;

        if self.angle.x< -3.14/2.0 {
            self.angle.x=-3.14/2.0;
        }

        if self.angle.x> 3.14/2.0 {
            self.angle.x=3.14/2.0;
        }

        self.calc_matrix();
    }

    fn on_mouse_wheel(&mut self, delta:MouseScrollDelta) {
        let scroll_y=match delta {
            MouseScrollDelta::LineDelta(x,y) =>
                y,
            _ => 0.0,
        };

        let old_distance=self.distance;
        let new_distance=if self.distance-scroll_y < 1.0 {
            1.0
        }else{
            self.distance-scroll_y
        };

        self.distance=new_distance;

        /*
        if old_distance.ceil()!=new_distance.ceil() {
            storage.grid.rebuild(new_distance, &window);
        }
        */

        self.calc_matrix();
    }

    fn calc_matrix(&mut self) {
        use cgmath::ApproxEq;
        use cgmath::Rotation;
        use cgmath::EuclideanSpace;

        let rot_x:Basis3<f32>=Rotation3::from_angle_x(Rad(self.angle.x));
        let rot_y:Basis3<f32>=Rotation3::from_angle_y(Rad(self.angle.y));
        let a=rot_x.rotate_vector(vec3(0.0,0.0,self.distance));
        let b=rot_y.rotate_vector(a);

        self.camera_position=Pos3D::from_vec(b+self.center_position.to_vec());
        self.camera_matrix=Matrix4::look_at(self.camera_position, self.center_position, vec3(0.0,1.0,0.0));
    }
}

impl Camera{
    pub fn new(window_width:u32, window_height:u32) -> Self {
        Camera {
            inner: Arc::new(Mutex::new(InnerCamera::new(window_width, window_height)))
        }
    }

    pub fn resize(&self, window_width:u32, window_height:u32) -> Result<(),Error> {
        mutex_lock!(self.inner => camera);
        camera.viewport=Viewport::configure(window_width, window_height);

        ok!()
    }

    pub fn rotate(&self, input:&Input) -> Result<(),Error> {
        mutex_lock!(self.inner => camera);
        camera.rotate(input);

        ok!()
    }

    pub fn on_mouse_wheel(&self, delta:MouseScrollDelta) -> Result<(),Error> {
        mutex_lock!(self.inner => camera);
        camera.on_mouse_wheel(delta);

        ok!()
    }

    pub fn get_render_camera(&self) -> Result<Option<render::Camera>,Error> {
        mutex_lock!(self.inner => camera);

        let perspective_matrix=match camera.viewport {
            Some( ref viewport ) => viewport.perspective_matrix.clone(),
            None => return ok!(None),
        };

        let render_camera=render::Camera::new(
            camera.camera_matrix.clone(),
            camera.camera_position.clone(),
            perspective_matrix
        );

        ok!( Some(render_camera) )
    }
}
