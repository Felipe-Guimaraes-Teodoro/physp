use chaos_framework::*;
use rapier3d::prelude::*;

use crate::{globals::{modify_rb_overhaul_size, read_rb_overhaul_size}, phys::{PhysicalWorld, PhysicsCommand, World}, selection::{update_selection_shader_from_renderer, SELECTION_SHADER}};

pub struct AppViewport {
    ctx: ViewportCtx,
    selected: usize,
}

#[derive(Clone)]
pub struct ViewportCtx {
    pub rb_size: f32,
    pub render_time: f32,
    pub phys_time: f32,

    pub dt: f32,

    pub w: i32,
    pub w_padding: i32,
    pub h: i32,
    pub h_padding: i32,

    pub edit_mode: bool,

    pub current_body_handle: Option<RigidBodyHandle>,
    pub current_body: Option<RigidBody>,

    pub selected: String,

    pub hierarchy: Option<RigidBodySet>,

    pub selection_mesh: MeshHandle,

    pub lmb: bool,
}

impl ViewportCtx {
    pub fn new(renderer: &mut Renderer) -> Self {
        let mut sphere = Sphere::new(16, 2.0, Vec4::ONE).mesh();
        sphere.shader = *SELECTION_SHADER;

        Self {
            rb_size: 1.0,
            render_time: 0.0,
            phys_time: 0.0,

            dt: 0.0,
            
            w: 500,
            w_padding: 200,
            h: 500,
            h_padding: 100,

            edit_mode: false,

            current_body: None,
            current_body_handle: None,

            selected: "Dynamic".to_string(),

            hierarchy: None,
            selection_mesh: renderer
                .add_mesh(sphere)
                .unwrap(),

            lmb: false,
        }
    }

    pub async fn update(
        &mut self, 
        world: &mut World, 
        renderer: &mut Renderer,
        el: &EventLoop,
        current_handle: Option<RigidBodyHandle>,
    ) {
        let phys_world = world.phys_world.lock().await;

        if let Some(handle) = current_handle {
            self.current_body_handle = Some(handle);
        }
        
        if let Some(handle) = self.current_body_handle {
            if let Some(body) = phys_world.rigid_body_set.get(handle) {
                self.current_body = Some(body.clone());
                let collider = body.colliders()[0];
                let mut total_force = Vec3::ZERO;

                for pair in phys_world.narrow_phase.contact_pairs_with(collider) {
                    let manifolds = &pair.manifolds;

                    for manifold in manifolds {
                        for point in &manifold.points {
                            let contact_point_impulse = point.data.impulse;  
                            let contact_normal = manifold.data.normal;            
            
                            let force = contact_point_impulse * contact_normal;
                            
                            let glam_force = vec3(force.x, force.y, force.z);

                            total_force += glam_force;
                        }
                    }

                    if let Some(phys_mesh_handle) = world.get_phys_mesh_from_handle(handle) {
                        let mesh_handle = world.phys_meshes[phys_mesh_handle].mesh;

                        let stress = total_force.length();

                        let color = vec3(stress, 0.0, 16.0 - stress) / 8.0 * read_rb_overhaul_size().cbrt();

                        renderer.meshes[mesh_handle].color = color;
                    }
                }

            }   
        }

        if el.event_handler.rmb {

        }
        
        self.hierarchy = Some(phys_world.rigid_body_set.clone());
    }
}

impl AppViewport {
    pub async fn update(
        ctx: &mut ViewportCtx, 
        el: &mut EventLoop, 
        renderer: &mut Renderer,
        world: &mut World,
    ) {
        update_selection_shader_from_renderer(renderer);

        if el.event_handler.key_just_pressed(glfw::Key::B) {
            ctx.edit_mode = !ctx.edit_mode;
        }

        ctx.lmb = el.event_handler.lmb;
        ctx.dt = el.dt;

        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!\nTIP: hold alt to toggle mouse mode");

        modify_rb_overhaul_size(ctx.rb_size);
        
        ctx.w = (el.event_handler.width - 200.0) as i32;
        ctx.h = (el.event_handler.height - 100.0) as i32;
        
        unsafe {
            if ctx.edit_mode {
                edit_gui(frame, ctx, renderer, world).await;
                renderer.camera.proj = Mat4::perspective_rh_gl(80.0f32.to_radians(), ctx.w as f32/ ctx.h as f32, 0.1, 1000.0);
                Viewport(0, 0, ctx.w, ctx.h);
            } else {
                let (w, h) = el.window.get_framebuffer_size();
                renderer.camera.proj = Mat4::perspective_rh_gl(80.0f32.to_radians(), el.event_handler.width/el.event_handler.height, 0.1, 1000.0);
                renderer.meshes[ctx.selection_mesh].position = Vec3::ONE * -2.0;
                Viewport(0, 0, w, h);
            }
        }

    }
}

pub async fn edit_gui(
    frame: &mut Ui,
    ctx: &mut ViewportCtx, 
    renderer: &mut Renderer,
    world: &mut World,
) {
    frame.show_default_style_editor();

    let mut body_pos = Vec3::ZERO;

    frame 
        .window("INFO")
        .collapsible(false)
        .resizable(false)
        .size([ctx.w as f32, ctx.h_padding as f32], Condition::Always)
        .position([0.0, 0.0], Condition::Always)
        .build(|| {
            frame.columns(5, "0", true);
            
            frame.text(format!("RT: {:.1}ms\nDT: {:.1}", ctx.render_time*1000.0, ctx.dt));
            

            if let Ok(status) = world.status {
                 let solve_time = status.solve_time*1000.0;
                 frame.text(format!("ST: {:.1}", solve_time));
            } else {
                frame.text("ST: UND.");
            }

            frame.next_column();

            frame.slider("RB_OVERHAUL_SIZE", 0.1, 10.0, &mut ctx.rb_size);
        });

        
    frame 
        .window("EXPLORER")
        .collapsible(false)
        .resizable(false)
        .size([ctx.w_padding as f32, (ctx.h + ctx.h_padding) as f32 / 2.0], Condition::Always)
        .position([ctx.w as f32, 0.0], Condition::Always)
        .build(|| {
            if let Some(hierarchy) = &ctx.hierarchy {
                // hierarchy.iter().for_each(|(handle, body)| {
                //     frame.text(format!("{:?}", body));
                // });
                frame.text(format!("{}", hierarchy.len()));
            }
        });

    frame 
        .window("PROPERTIES")
        .collapsible(false)
        .size([ctx.w_padding as f32, (ctx.h + ctx.h_padding) as f32 / 2.0], Condition::Always)
        .position([ctx.w as f32, (ctx.h + ctx.h_padding) as f32 / 2.0], Condition::Always)
        .build(|| {
            let mut pos = Vec3::ONE * -2.0;
            if let Some(body) = &mut ctx.current_body {
                frame.text(format!("GAV. POT. ENERGY: {:.1}", body.gravitational_potential_energy(0.16, vector![0.0, -9.81, 0.0])));
                frame.text(format!("LIN. VELOCITY: {:.1}", body.linvel()));

                let items = vec!["Dynamic"];
                
                if let Some(_cb) = frame.begin_combo("RB TYPE", format!("{:?}", body.body_type())) {
                    for cur in &items {
                        if &ctx.selected.as_str() == cur {
                            frame.set_item_default_focus();
                        }
                        let clicked = frame.selectable_config(cur)
                            .selected(&ctx.selected.as_str() == cur)
                            .build();
                        if clicked {
                            ctx.selected = cur.to_string();
                        }
                    }
                }

                pos = vec3(body.translation().x, body.translation().y, body.translation().z);
                
                body_pos = vec3(pos.x, pos.y, pos.z);
            } else {
                
            }
            renderer.meshes[ctx.selection_mesh].position = lerp(renderer.meshes[ctx.selection_mesh].position, vec3(pos.x, pos.y, pos.z), 0.125);
            renderer.meshes[ctx.selection_mesh].scale = Vec3::ONE * read_rb_overhaul_size();
        });

    if let Some(handle) = ctx.current_body_handle {
        if ctx.lmb {
            world.command_sender
                .try_send(PhysicsCommand::Impulse(renderer.camera.pos - body_pos, handle))
                .unwrap();
        }
    }
}