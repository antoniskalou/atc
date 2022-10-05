use crate::geom::Point;

const MIN_ZOOM: f32 = 0.00001;

#[derive(Clone, Debug)]
pub struct Camera {
    screen_size: glm::Vec2,
    view_center: glm::Vec2,
    view_size: glm::Vec2,
    zoom: f32,
}

impl Camera {
    pub fn new(screen_width: f32, screen_height: f32, view_width: f32, view_height: f32) -> Self {
        Self {
            screen_size: glm::vec2(screen_width, screen_height),
            view_size: glm::vec2(view_width, view_height),
            view_center: glm::zero(),
            zoom: 1.,
        }
    }

    pub fn screen_size(&self) -> glm::Vec2 {
        self.screen_size
    }

    pub fn move_by(&mut self, offset: glm::Vec2) {
        self.view_center += offset;
    }

    pub fn move_to(&mut self, point: glm::Vec2) {
        self.view_center = point;
    }

    /// zoom the camera by a factor, e.g. 0.5 zooms out, 2.0 zooms in
    pub fn zoom(&mut self, scale: f32) {
        // FIXME: can return 0 if small enough, maybe just use a zoom scalar
        self.zoom = (self.zoom * scale).max(MIN_ZOOM);
    }

    pub fn world_to_screen_coords(&self, point: glm::Vec2) -> Point {
        let pixels_per_unit = self.pixels_per_unit();
        let view_offset = point - self.view_center;
        let view_scale = view_offset.component_mul(&pixels_per_unit);

        let x = view_scale.x + self.screen_size.x / 2.0;
        let y = self.screen_size.y - (view_scale.y + self.screen_size.y / 2.0);
        Point { x, y }
    }

    pub fn pixels_per_unit(&self) -> glm::Vec2 {
        self.screen_size.component_div(&self.view_size) * self.zoom
    }
}
