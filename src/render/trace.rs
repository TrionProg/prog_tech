use nes::{ErrorInfo,ErrorInfoTrait};

use types::*;

use object_pool::growable::Pool;

use super::Error;
use super::Storage;
use super::Encoder;
use super::Targets;

pub struct Trace {
    x:u32,
    z:u32,
    angle:f32,
    color:[f32;4],
    mesh_id:TraceMeshID
}

impl Trace {
    pub fn new(
        x:u32,
        z:u32,
        angle:f32,
        color:[f32;4],
        mesh_id:TraceMeshID
    ) -> Self {
        Trace {
            x,
            z,
            angle,
            color,
            mesh_id
        }
    }
}

pub struct TracePool {
    pool:Pool<Trace, Trace>,
}

impl TracePool {
    pub fn new() -> Self {
        TracePool {
            pool:Pool::new()
        }
    }

    pub fn insert(&mut self, trace:Trace) {
        self.pool.insert(trace);
    }

    pub fn delete(&mut self,id:TraceID) {
        self.pool.remove(id.get_id());
    }

    pub fn draw(&self, storage:&Storage, encoder:&mut Encoder, targets:&Targets ) -> Result<(),Error> {
        for trace in self.pool.iter() {
            storage.trace_meshes.get(trace.mesh_id)?.draw(
                storage, encoder, targets,
                trace.x, trace.z, trace.angle, trace.color
            )?;
        }

        ok!()
    }
}