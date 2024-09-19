use chaos_framework::{vec3, EventLoop, Renderer, Vec3};

use crate::globals::read_rb_overhaul_size;
use crate::{phys::World, raycaster::Raycaster, viewport::ViewportCtx};

use crate::phys::PhysicsCommand;

pub struct RbBuilderCtx {

}

pub struct RbBuilder {

}

impl RbBuilder {
    pub async fn update(world: &mut World, renderer: &mut Renderer, el: &EventLoop, ctx: &ViewportCtx) {
        if el.event_handler.rmb {
            if let Some(pos) = Raycaster::get_world_pos_from_mouse(&el, renderer, world, ctx).await {
                add_cube(world, renderer, pos).await;
            }
        }

        if el.event_handler.key_just_pressed(glfw::Key::F) {
            if let Some(pos) = Raycaster::get_world_pos_from_mouse(&el, renderer, world, ctx).await {
                add_cube(world, renderer, pos).await;
            }
        }
    }   
}

pub async fn add_cube(world: &mut World, renderer: &mut Renderer, pos: Vec3) {
    let cube = world.add_cube(renderer).await;
    let handle = world.phys_meshes[cube].body;

    world.command_sender.send(PhysicsCommand::Translate(pos + vec3(0.0, read_rb_overhaul_size(), 0.0), handle)).await.unwrap();
}