use nalgebra::Vector2;

pub trait UiComponent {
    fn set_dimensions(&mut self, dimensions:Vector2<f32>);

    fn dimensions(&self) -> Vector2<f32>;

    fn set_position(&mut self, position:Vector2<f32>);

    fn position(&self) -> Vector2<f32>;
}