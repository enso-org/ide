use nalgebra::Vector3;

// FIXME: This really needs to be refactored somewhere. It can be done in the next commit but there sohuld be a note about it.
pub trait HasPosition {
    /// Gets self's position.
    fn position(&self) -> Vector3<f32>;
    /// Sets self's position.
    fn set_position(&mut self, position:Vector3<f32>);
}
