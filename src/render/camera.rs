
use cgmath::{Vector2};
use location::*;

pub struct Camera {
    pub camera_matrix:Matrix4,
    pub camera_position:Pos3D,

    pub perspective_matrix:Matrix4,
}

impl Camera {
    pub fn new(
        camera_matrix:Matrix4,
        camera_position:Pos3D,
        perspective_matrix:Matrix4
    ) -> Self {
        Camera {
            camera_matrix,
            camera_position,
            perspective_matrix
        }
    }
}