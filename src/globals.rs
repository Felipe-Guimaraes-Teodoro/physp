use std::sync::{Arc, LazyLock, Mutex};

pub static RB_OVERHAUL_SIZE: LazyLock<Arc<Mutex<f32>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(1.0))
});

pub fn modify_rb_overhaul_size(val: f32) {
    *RB_OVERHAUL_SIZE.lock().unwrap() = val;
}

pub fn read_rb_overhaul_size() -> f32 {
    *RB_OVERHAUL_SIZE.lock().unwrap()
}