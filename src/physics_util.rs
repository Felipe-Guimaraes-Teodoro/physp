use chaos_framework::{quat, vec3, Cuboid, MeshHandle, Renderer, Sphere, Vec3, Vec4};
use rapier3d::{parry::query::Ray, prelude::*};

use crate::{globals::read_rb_overhaul_size, phys::{self, PhysMeshHandle, World}};

impl phys::PhysicalWorld {
    pub fn add_floor(&mut self, size: Vec3) -> ColliderHandle {
        let ground_collider = ColliderBuilder::cuboid(size.x, size.y, size.z)
            .translation(vector![0.0, -size.y, 0.0])
            .build();
        self.collider_set.insert(ground_collider)
    }

    pub fn add_sphere_rigidbody(&mut self, x: f32, y: f32, z: f32, r: f32) -> RigidBodyHandle {
        let r = read_rb_overhaul_size();
        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .build();
        let collider = ColliderBuilder::ball(r).restitution(0.7).friction(0.5).build();
        let body_handle = self.rigid_body_set.insert(rb.clone());

        self.collider_set.insert_with_parent(collider.clone(), body_handle, &mut self.rigid_body_set);

        return body_handle;
    }

    pub fn add_cube_rigidbody(&mut self, x: f32, y: f32, z: f32, r: f32) -> RigidBodyHandle {
        let r = read_rb_overhaul_size();
        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .build();
        let collider = ColliderBuilder::cuboid(r, r, r).restitution(0.3).friction(0.5).build();
        let body_handle = self.rigid_body_set.insert(rb.clone());

        self.collider_set.insert_with_parent(collider.clone(), body_handle, &mut self.rigid_body_set);

        return body_handle;
    }

    pub fn body_raycast(&mut self, origin: Vec3, direction: Vec3) -> Option<RigidBodyHandle> {
        let ray = Ray::new(
            vector![origin.x, origin.y, origin.z].into(), 
            vector![direction.x, direction.y, direction.z]
        );

        if let Some((handle, _hit)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,          
            &ray,               
            1000.0,            
            true,   
            QueryFilter::default() 
        ) {
            if let Some(collider) = self.collider_set.get(handle) {
                return collider.parent();
            } else {
                return None;
            }
        } else {
            return None;
        };
    }
    
    pub fn pos_raycast(&mut self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let ray = Ray::new(
            vector![origin.x, origin.y, origin.z].into(), 
            vector![direction.x, direction.y, direction.z]
        );

        if let Some((_handle, dist)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,          
            &ray,               
            1000.0,            
            true,   
            QueryFilter::default() 
        ) {
            return Some(origin + (direction * dist));
        } else {
            return None;
        };
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
        let mut sphere_mesh = Sphere::new(16, read_rb_overhaul_size(), Vec4::ONE).mesh();
        for face in sphere_mesh.indices.chunks_mut(3) {
            face.reverse();
        }
        Self {
            mesh: renderer.add_mesh(
                sphere_mesh
            ).unwrap(),
            body: phys_world.add_sphere_rigidbody(0.0, 1.0, 0.0, 1.0)
        }
    }

    pub fn cube(renderer: &mut Renderer, phys_world: &mut phys::PhysicalWorld) -> Self {
        let mut cube_mesh = Cuboid::new(Vec3::ONE * 2.0 * read_rb_overhaul_size(), Vec4::ONE).mesh();
        for face in cube_mesh.indices.chunks_mut(3) {
            face.reverse();
        }
        Self {
            mesh: renderer.add_mesh(
                cube_mesh
            ).unwrap(),
            body: phys_world.add_cube_rigidbody(0.0, 1.0, 0.0, 1.0)
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
    pub async fn add_sphere(&mut self, renderer: &mut Renderer) -> PhysMeshHandle {
        let mut phys_world = self.phys_world.lock().await;
        let sphere = PhysMesh::sphere(renderer, &mut phys_world);
        let handle = PhysMeshHandle {
            id: self.phys_meshes.len() as u32,
        };

        self.phys_meshes.insert(handle, sphere);

        handle
    }

    pub fn get_phys_mesh_from_handle(&self, handle: RigidBodyHandle) -> Option<PhysMeshHandle> {
        for (phys_mesh_handle, phys_mesh) in &self.phys_meshes {
            if phys_mesh.body == handle {
                return Some(*phys_mesh_handle);
            }
        }

        return None;
    }

    pub async fn add_cube(&mut self, renderer: &mut Renderer) -> PhysMeshHandle {
        let mut phys_world = self.phys_world.lock().await;
        let cube = PhysMesh::cube(renderer, &mut phys_world);
        let handle = PhysMeshHandle {
            id: self.phys_meshes.len() as u32,
        };

        self.phys_meshes.insert(handle, cube);

        handle
    }

    pub fn add_floor(&mut self, size: Vec3) {
        if let Ok(mut phys_world) = self.phys_world.try_lock() {
            phys_world.add_floor(size);
        }
    }

    pub async fn destroy(&mut self, renderer: &mut Renderer, handle: PhysMeshHandle) {
        let mut phys_world = self.phys_world.lock().await;
        
        let phys_mesh = &self.phys_meshes[handle];
        phys_world.remove_rigidbody(phys_mesh.body);
        renderer.destroy_mesh(phys_mesh.mesh);
        self.phys_meshes.remove(&handle);
    }
}