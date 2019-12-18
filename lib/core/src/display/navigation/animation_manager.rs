// ========================
// === AnimationManager ===
// ========================

pub struct AnimationManager {
    maximum_frame_duration: f32
}

impl AnimationManager {
    pub fn new(minimum_fps : f32) -> Self {
        let maximum_frame_duration = 1.0 / minimum_fps;
        Self { maximum_frame_duration }
    }

    /// Runs the closure guaranteeing it runs at `minimum_fps`.
    pub fn run<F:FnMut(f32)>(&mut self, mut dt:f32, mut f:F) {
        while dt > self.maximum_frame_duration {
            f(self.maximum_frame_duration);
            dt -= self.maximum_frame_duration;
        }
        f(dt);
    }
}