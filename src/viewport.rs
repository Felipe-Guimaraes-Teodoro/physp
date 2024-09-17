use chaos_framework::*;
use rapier3d::prelude::*;

use crate::{globals::modify_rb_overhaul_size, phys::PhysicalWorld};

pub struct AppViewport {
    ctx: ViewportCtx,
}

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

    pub hierarchy: Option<RigidBodySet>,
}

impl ViewportCtx {
    pub fn new() -> Self {
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

            hierarchy: None,
        }
    }

    pub fn update(&mut self, phys_world: &mut PhysicalWorld) {
        if let Some(handle) = self.current_body_handle {
            self.current_body = Some(phys_world.rigid_body_set[handle].clone());
        }

        self.hierarchy = Some(phys_world.rigid_body_set.clone());
    }
}

impl AppViewport {
    pub fn uptate(ctx: &mut ViewportCtx, el: &mut EventLoop, renderer: &mut Renderer) {
        if el.event_handler.key_just_pressed(glfw::Key::B) {
            ctx.edit_mode = !ctx.edit_mode;
        }

        ctx.dt = el.dt;

        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!\nTIP: hold alt to toggle mouse mode");

        modify_rb_overhaul_size(ctx.rb_size);
        
        ctx.w = (el.event_handler.width - 200.0) as i32;
        ctx.h = (el.event_handler.height - 100.0) as i32;
        
        unsafe {
            if ctx.edit_mode {
                edit_gui(frame, ctx);
                renderer.camera.proj = Mat4::perspective_rh_gl(80.0f32.to_radians(), ctx.w as f32/ ctx.h as f32, 0.1, 1000.0);
                Viewport(0, 0, ctx.w, ctx.h);
            } else {
                let (w, h) = el.window.get_framebuffer_size();
                renderer.camera.proj = Mat4::perspective_rh_gl(80.0f32.to_radians(), el.event_handler.width/el.event_handler.height, 0.1, 1000.0);
                Viewport(0, 0, w, h);
            }
        }
    }
}

pub fn edit_gui(frame: &mut Ui, ctx: &mut ViewportCtx) {
    frame.show_default_style_editor();

    frame 
        .window("INFO")
        .collapsible(false)
        .resizable(false)
        .size([ctx.w as f32, ctx.h_padding as f32], Condition::Always)
        .position([0.0, 0.0], Condition::Always)
        .build(|| {
            frame.columns(5, "0", true);
            
            frame.text(format!("RT: {:.1}ms\nST: {:.1}ms\nDT: {:.1}", ctx.render_time*1000.0, ctx.phys_time*1000.0, ctx.dt));
            
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
                hierarchy.iter().for_each(|(handle, body)| {
                    frame.text(format!("{:?}", body));
                });
            }
        });

    frame 
        .window("PROPERTIES")
        .collapsible(false)
        .size([ctx.w_padding as f32, (ctx.h + ctx.h_padding) as f32 / 2.0], Condition::Always)
        .position([ctx.w as f32, (ctx.h + ctx.h_padding) as f32 / 2.0], Condition::Always)
        .build(|| {
            if let Some(body) = &mut ctx.current_body {
                frame.text(format!("GAV. POT. ENERGY: {:.1}", body.gravitational_potential_energy(0.16, vector![0.0, -9.81, 0.0])));
                frame.text(format!("LIN. VELOCITY: {:.1}", body.linvel()));
                frame.text(format!("TY: {:?}", body.body_type()))
            }
        });
}