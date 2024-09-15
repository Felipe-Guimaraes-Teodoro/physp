mod phys;
mod physics_util;
mod server;
mod client;

use std::sync::{Arc, Mutex};

use chaos_framework::*;
use client::Client;
use glfw::Key;
use phys::{PhysMeshHandle, World};
use rapier3d::{prelude::*, rayon::iter::{IntoParallelIterator, ParallelIterator}};
use server::Server;
use tokio::task;

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

    el.window.glfw.set_swap_interval(SwapInterval::Sync(0));

    let mut world = World::new();
    world.phys_world.add_floor(vec3(125.0, 0.2, 125.0));

    let addr = "127.0.0.1:4040";
    let mut server = Server::new(addr).await.unwrap();

    let mut client = Client::new().await.unwrap();

    task::spawn(async move {
        server.run().await.expect("server failed to run");
    });
    
    client.send_message(addr, "hi!").await.unwrap();

    while !el.window.should_close() {
        el.update();
        renderer.update();
        world.update(&mut renderer, el.dt * 2.0).await;
        
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);

        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!\nTIP: hold alt to toggle mouse mode");
        
        if el.is_key_down(Key::LeftAlt) {
            el.window.set_cursor_mode(CursorMode::Normal);
        } else {
            el.window.set_cursor_mode(CursorMode::Disabled);
        }

        if el.event_handler.key_just_pressed(Key::F) {
            gen_spheres(&mut world, &mut renderer);
        }

        let handles: Vec<PhysMeshHandle> = world.phys_meshes.iter().map(|v| *v.0).collect();
        if el.event_handler.key_just_pressed(Key::R) {
            for handle in handles {
                world.destroy(&mut renderer, handle);
            }
        }

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            renderer.camera.proj = Mat4::perspective_rh_gl(70.0, 12.0/9.0, 0.1, 1000.0);
            renderer.draw();
            el.ui.draw();
        }

    }
}

pub fn gen_spheres(world: &mut World, renderer: &mut Renderer) {
    let world = Arc::new(Mutex::new(world));
    let renderer = Arc::new(Mutex::new(renderer));

    (0..128).into_par_iter().for_each(|_| {
        let mut world = world.lock().unwrap();
        let mut renderer = renderer.lock().unwrap();

        let sphere = world.add_sphere(&mut *renderer);

        let body_handle = {
            let phys_mesh = &world.phys_meshes[sphere];
            phys_mesh.body
        };

        world.phys_world.rigid_body_set[body_handle]
            .set_position(vector![rand_betw(-2.0, 2.0), rand_betw(0.0, 4.0), rand_betw(-2.0, 2.0)].into(), false);
    });
    
}