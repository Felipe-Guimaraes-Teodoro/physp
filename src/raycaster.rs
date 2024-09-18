use chaos_framework::{vec2, EventLoop, Renderer};
use rapier3d::prelude::{RigidBody, RigidBodyHandle};

use crate::{phys::World, utils::get_ray_from_mouse, viewport::ViewportCtx};

pub struct Raycaster {

}

impl Raycaster {
    pub async fn get_body_from_mouse_pos(
        el: &EventLoop, 
        renderer: &Renderer,
        world: &mut World,
        ctx: &ViewportCtx,
    ) -> Option<RigidBodyHandle> {
        let mouse_pos = el.event_handler.mouse_pos;
        let (w, h) = el.window.get_size();
        let window_size = vec2(w as f32, h as f32);
        let projection = renderer.camera.proj;
        let view = renderer.camera.view;

        let (origin, dir) = get_ray_from_mouse(
            mouse_pos, 
            window_size, 
            projection, 
            ctx.w_padding as f32,
            ctx.h_padding as f32,
            view
        );

        let mut phys_world = world.phys_world.lock().await;
        
        if let Some(handle) = phys_world.body_raycast(origin, dir) {
            return Some(handle)
        } else {
            return None;
        }
    }
}