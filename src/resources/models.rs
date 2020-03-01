use raylib::{
    models::{Mesh as RMesh, Model as RModel},
    RaylibHandle, RaylibThread,
};

// -----------------------------------------------------------------------------

use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Model(pub ManuallyDrop<RModel>); // TODO(cmc): explain

unsafe impl Send for Model {} // TODO(cmc): explain
unsafe impl Sync for Model {} // TODO(cmc): explain

impl Deref for Model {
    type Target = RModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Model {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Mesh(pub ManuallyDrop<RMesh>); // TODO(cmc): explain

unsafe impl Send for Mesh {} // TODO(cmc): explain
unsafe impl Sync for Mesh {} // TODO(cmc): explain

impl Deref for Mesh {
    type Target = RMesh;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Mesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MeshID(usize);

#[derive(Debug, Default)]
pub struct MeshStore {
    meshes: Vec<Mesh>,
}

impl MeshStore {
    pub const CUBE: MeshID = MeshID(0);
    pub const PLANE: MeshID = MeshID(1);

    // TODO(cmc): better/safer init
    pub fn new(rl_thread: &RaylibThread) -> Self {
        #[rustfmt::skip]
        let meshes = vec![
            Mesh(ManuallyDrop::new(RMesh::gen_mesh_cube(rl_thread, 1., 1., 1.))),
            Mesh(ManuallyDrop::new(RMesh::gen_mesh_cube(rl_thread, 1., 0., 1.))),
        ];

        Self { meshes }
    }

    pub fn instantiate_model(
        &self,
        rl: &mut RaylibHandle,
        rl_thread: &RaylibThread,
        mesh_id: MeshID,
    ) -> Model {
        let mesh = &self.meshes[mesh_id.0];
        let model = rl.load_model_from_mesh(rl_thread, &mesh).unwrap();

        Model(ManuallyDrop::new(model))
    }
}
