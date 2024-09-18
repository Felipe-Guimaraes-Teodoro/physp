use chaos_framework::{vec3, vec4, Mat4, Vec2, Vec3};

pub fn get_ray_from_mouse(
    mouse_pos: Vec2, 
    window_size: Vec2,
    projection: Mat4,
    padding_x: f32,
    padding_y: f32,
    view: Mat4,
) -> (Vec3, Vec3) {
    let ndc_x = (2.0 * (mouse_pos.x + padding_x / 2.0)) / window_size.x;
    let ndc_y = (2.0 * (mouse_pos.y + padding_y / 2.0)) / window_size.y;
    let ndc = vec3(ndc_x, ndc_y, 1.0);

    let inv_projection = projection.inverse();
    let inv_view = view.inverse();

    let clip_coords = vec4(ndc.x, ndc.y, ndc.z, 1.0);

    let eye_coords = inv_projection * clip_coords;
    let eye_coords = vec4(eye_coords.x, eye_coords.y, -1.0, 0.0);

    let world_coords = inv_view * eye_coords;

    let ray_origin = inv_view.col(3).truncate();
    let ray_direction = world_coords.truncate().normalize();

    (ray_origin, ray_direction)
}
