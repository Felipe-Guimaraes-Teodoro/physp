use chaos_framework::{vec3, vec4, Mat4, Vec2, Vec3};

pub fn get_ray_from_mouse(
    mouse_pos: Vec2, 
    window_size: Vec2,
    projection: Mat4,
    view: Mat4,
) -> (Vec3, Vec3) {
    let ndc_x = (2.0 * mouse_pos.x) / window_size.x - 1.0;
    let ndc_y = 1.0 - (2.0 * mouse_pos.y) / window_size.y;

    let ndc = vec3(ndc_x, ndc_y, 1.0);

    let inv_proj_view = (projection * view).inverse();

    let clip_coords = vec4(ndc.x, ndc.y, -1.0, 1.0);

    let world_coords = inv_proj_view * clip_coords;
    let world_coords = world_coords.truncate() / world_coords.w;

    let ray_origin = view.inverse().col(3).truncate();
    let ray_direction = (world_coords - ray_origin).normalize();

    (ray_origin, ray_direction)
}