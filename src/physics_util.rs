use chaos_framework::{quat, vec3, MeshHandle, Renderer, Sphere, Vec3, Vec4};
use rapier3d::prelude::*;

use crate::phys::{self, PhysMeshHandle, World};

impl phys::PhysicalWorld {
    pub fn add_floor(&mut self, size: Vec3) -> ColliderHandle {
        let ground_collider = ColliderBuilder::cuboid(size.x, size.y, size.z)
            .translation(vector![0.0, -size.y, 0.0])
            .build();
        self.collider_set.insert(ground_collider)
    }

    pub fn add_sphere_rigidbody(&mut self, x: f32, y: f32, z: f32, r: f32) -> RigidBodyHandle {
        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .build();
        let collider = ColliderBuilder::ball(r).restitution(0.7).friction(0.5).build();
        let body_handle = self.rigid_body_set.insert(rb.clone());

        self.collider_set.insert_with_parent(collider.clone(), body_handle, &mut self.rigid_body_set);

        return body_handle;
    }

    pub fn remove_rigidbody(&mut self, handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle, 
            &mut self.island_manager, 
            &mut self.collider_set, 
            &mut self.impulse_joint_set, 
            &mut self.multibody_joint_set, 
            true,
        );
    }
}

pub struct PhysMesh {
    pub mesh: MeshHandle,
    pub body: RigidBodyHandle,
}

impl PhysMesh {
    pub fn sphere(renderer: &mut Renderer, phys_world: &mut phys::PhysicalWorld) -> Self {
        Self {
            mesh: renderer.add_mesh(
                Sphere::new(16, 1.0, Vec4::ONE).mesh()
            ).unwrap(),
            body: phys_world.add_sphere_rigidbody(0.0, 1.0, 0.0, 1.0)
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer, phys_world: &mut phys::PhysicalWorld) {
                        // once told me the world is gonna roll me
        if let Some(body) = phys_world.rigid_body_set.get_mut(self.body) {
            let pos = body.translation();
            let rot = body.rotation();
    
            renderer.meshes[self.mesh].position = vec3(pos.x, pos.y, pos.z);
            renderer.meshes[self.mesh].rotation = quat(rot.i, rot.j, rot.k, rot.w);
        }
    }
}

impl World {
    pub fn add_sphere(&mut self, renderer: &mut Renderer) -> PhysMeshHandle {
        let sphere = PhysMesh::sphere(renderer, &mut self.phys_world);
        let handle = PhysMeshHandle {
            id: self.phys_meshes.len() as u32,
        };

        self.phys_meshes.insert(handle, sphere);

        handle
    }

    pub fn destroy(&mut self, renderer: &mut Renderer, handle: PhysMeshHandle) {
        let phys_mesh = &self.phys_meshes[handle];
        self.phys_world.remove_rigidbody(phys_mesh.body);
        renderer.destroy_mesh(phys_mesh.mesh);
        self.phys_meshes.remove(&handle);
    }
}