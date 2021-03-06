
use object_pool::growable::ID;

pub trait LodID {
    fn new(id:ID) -> Self;
    fn get_id(&self) -> ID;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ObjectLodID(ID);

impl LodID for ObjectLodID {
    fn new(id:ID) -> Self {ObjectLodID(id)}
    fn get_id(&self) -> ID {self.0}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TraceLodID(ID);

impl LodID for TraceLodID {
    fn new(id:ID) -> Self {TraceLodID(id)}
    fn get_id(&self) -> ID {self.0}
}