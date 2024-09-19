use std::{collections::HashMap, ops::{Index, IndexMut}, sync::{mpsc::channel, Arc}};

use chaos_framework::{Renderer, Vec3};
use rapier3d::prelude::*;
use tokio::sync::{mpsc::{self, error::TryRecvError, Receiver, Sender}, Mutex};

use crate::{globals::read_rb_overhaul_size, physics_util::PhysMesh};

/* TODO: add the physics meshes here to grant access to meshes */
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

    pub fn step(&mut self, dt: f32) {
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

pub enum PhysicsCommand {
    Impulse(Vec3, RigidBodyHandle),
    SetType(RigidBodyType, RigidBodyHandle),
    Translate(Vec3, RigidBodyHandle),
}

#[derive(Copy, Clone)]
pub struct PhyisicsStatus {
    pub solve_time: f32,
}

pub struct World {
    pub phys_world: Arc<Mutex<PhysicalWorld>>,
    pub phys_meshes: HashMap<PhysMeshHandle, PhysMesh>,
    dt_sender: Sender<f32>,
    pub command_sender: Sender<PhysicsCommand>,
    pub report_receiver: Receiver<PhyisicsStatus>,
    pub status: Result<PhyisicsStatus, TryRecvError>,
}

impl World {
    pub async fn new() -> Self {
        let (dt_sender, mut dt_receiver) = mpsc::channel(1);
        let (command_sender, mut command_receiver) = mpsc::channel(16);
        let (report_sender, mut report_receiver) = mpsc::channel(16);

        let phys_world = Arc::new(Mutex::new(PhysicalWorld::new()));
        let phys_world_clone = phys_world.clone();
        tokio::task::spawn(async move {
            let mut elapsed = 0.0;

            while let Some(dt) = dt_receiver.recv().await {
                if let Ok(mut phys_world) = phys_world_clone.try_lock() {
                    let now = std::time::Instant::now();
                    phys_world.step(dt);
                    elapsed = now.elapsed().as_secs_f32();

                    if let Ok(command) = command_receiver.try_recv() {
                        match command {
                            PhysicsCommand::Impulse(v, rigid_body_handle) => {
                                let body = &mut phys_world.rigid_body_set[rigid_body_handle];
                                body.apply_impulse(vector![v.x, v.y, v.z], true);
                            }
                            PhysicsCommand::SetType(rigid_body_type, rigid_body_handle) => {
                                let body = &mut phys_world.rigid_body_set[rigid_body_handle];
                                body.set_body_type(rigid_body_type, false);
                            }
                            PhysicsCommand::Translate(v, rigid_body_handle) => {
                                let body = &mut phys_world.rigid_body_set[rigid_body_handle];
                                body.set_position(vector![v.x, v.y, v.z].into(), false);
                            }
                        }
                    }

                    // std::thread::sleep_ms(16);

                    report_sender.try_send(
                        PhyisicsStatus {
                            solve_time: elapsed,
                        }
                    ).unwrap();
                }
            };
        });

        let status = report_receiver.try_recv();

        Self { phys_world, phys_meshes: HashMap::new(), dt_sender, command_sender, report_receiver, status, }
    }

    pub async fn update(&mut self, renderer: &mut Renderer, dt: f32) {
        /* TODO: every N frames, force the simulation to synchronize */
        if let Ok(mut phys_world) = self.phys_world.try_lock() {
            for phys_mesh in self.phys_meshes.values_mut() {
                phys_mesh.update(renderer, &mut phys_world);
            }
        }

        self.status = self.report_receiver.try_recv();

        self.dt_sender.send(dt).await.unwrap();
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