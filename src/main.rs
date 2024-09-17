mod phys;
mod physics_util;
mod server;
mod client;
mod globals;
mod viewport;
mod raycaster;
mod utils;

use std::sync::{Arc, Mutex};

use chaos_framework::*;
use client::Client;
use glfw::Key;
use globals::modify_rb_overhaul_size;
use phys::{PhysMeshHandle, World};
use rapier3d::{prelude::*, rayon::iter::{IntoParallelIterator, ParallelIterator}};
use raycaster::Raycaster;
use server::Server;
use tokio::task;
use viewport::{AppViewport, ViewportCtx};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let mut el = EventLoop::new(1200, 900);
    let mut renderer = Renderer::new();

    el.window.glfw.set_swap_interval(SwapInterval::None);
    
    unsafe {
        Enable(DEPTH_TEST);
        Enable(CULL_FACE);
    }

    let mut floor = Quad::new(vec3(250.0, 250.0, 250.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, -3.1415 * 0.5, 0.0, 0.0);
    floor.position = vec3(-125.0, 0.0, 125.0);
    floor.color = vec3(0.6, 0.6, 0.9);
    renderer.add_mesh(floor).unwrap();

    renderer.add_light(Light { position: Vec3::ONE, color: Vec3::ONE });

    el.window.glfw.set_swap_interval(SwapInterval::Sync(1));

    let mut world = World::new();

    world.add_floor(vec3(125.0, 0.2, 125.0));

    let addr = "127.0.0.1:4040";
    let mut server = Server::new(addr).await.unwrap();

    let mut client = Client::new().await.unwrap();

    task::spawn(async move {
        server.run().await.expect("server failed to run");
    });
    
    client.send_message(addr, "hi!").await.unwrap();

    let mut ctx = ViewportCtx::new();

    let mut current_handle = None;

    while !el.window.should_close() {
        el.update();
        renderer.update();
        let now = std::time::Instant::now();
        world.update(&mut renderer, el.dt).await;
        ctx.phys_time = now.elapsed().as_secs_f32();
        
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);

        {
            AppViewport::uptate(&mut ctx, &mut el, &mut renderer);
        }
        
        if el.is_key_down(Key::LeftAlt) {
            el.window.set_cursor_mode(CursorMode::Normal);
        } else {
            el.window.set_cursor_mode(CursorMode::Disabled);
        }

        if el.event_handler.key_just_pressed(Key::Q) {
            current_handle = Raycaster::get_body_from_mouse_pos(&el, &renderer, &mut world).await;
        }
        
        let handles: Vec<PhysMeshHandle> = world.phys_meshes.iter().map(|v| *v.0).collect();
        if el.event_handler.key_just_pressed(Key::R) {
            current_handle = None;
            for handle in handles {
                world.destroy(&mut renderer, handle).await;
            }
        }

        if el.event_handler.key_just_pressed(Key::F) {
            gen_spheres(&mut world, &mut renderer).await;
        }

        if el.event_handler.key_just_pressed(Key::J) {
            let _ = world.phys_world.lock().await; // force sync
        }

        if (el.time * 1000.0) as i32 % 8 == 0 {
            if let Some(handle) = current_handle {
                ctx.current_body_handle = Some(handle);

                let mut phys_world = world.phys_world.lock().await;
                ctx.update(&mut phys_world);
            }
        }

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            let now = std::time::Instant::now();
            renderer.draw();
            ctx.render_time = now.elapsed().as_secs_f32();
            el.ui.draw();
        }

    }
}

pub async fn gen_spheres(world: &mut World, renderer: &mut Renderer) {
    for i in 0..128 {
        let cube = world.add_cube(renderer).await;
        let handle = world.phys_meshes[cube].body;
        world.phys_world.lock().await.rigid_body_set.get_mut(handle).unwrap()
            .set_translation(vector![rand_betw(-2.0, 2.0), rand_betw(0.0, 4.0), rand_betw(-2.0, 2.0)], false);
    }
}
