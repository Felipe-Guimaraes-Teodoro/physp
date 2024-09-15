use std::{collections::HashMap, ops::{Index, IndexMut}};

use chaos_framework::Renderer;
use rapier3d::prelude::*;

use crate::physics_util::PhysMesh;

pub struct PhysicalWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl PhysicalWorld {
    pub fn new() -> Self {
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();

        Self {
            rigid_body_set,
            collider_set,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            physics_hooks,
            event_handler,
        }
    }

    pub async fn step(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;

        self.physics_pipeline.step(
            &vector![0.0, -9.81, 0.0], // gravity
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );
    }

}

pub struct World {
    pub phys_world: PhysicalWorld,
    pub phys_meshes: HashMap<PhysMeshHandle, PhysMesh>,
}

impl World {
    pub fn new() -> Self {
        Self { phys_world: PhysicalWorld::new(), phys_meshes: HashMap::new() }
    }

    pub async fn update(&mut self, renderer: &mut Renderer, dt: f32) {
        for phys_mesh in self.phys_meshes.values_mut() {
            phys_mesh.update(renderer, &mut self.phys_world);
        }

        self.phys_world.step(dt).await;
    }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct PhysMeshHandle {
    pub id: u32,
}

impl Index<PhysMeshHandle> for HashMap<PhysMeshHandle, PhysMesh> {
    type Output = PhysMesh;

    fn index(&self, handle: PhysMeshHandle) -> &Self::Output {
        self.get(&handle).expect("no entry for key")
    }
}

impl IndexMut<PhysMeshHandle> for HashMap<PhysMeshHandle, PhysMesh> {
    fn index_mut(&mut self, handle: PhysMeshHandle) -> &mut Self::Output {
        self.get_mut(&handle).expect("no entry for key")
    }
}