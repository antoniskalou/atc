use crate::geom::Point;

const MIN_ZOOM: f32 = 0.00001;

#[derive(Clone, Debug)]
pub struct Camera {
    screen_size: Point,
    view_center: Point,
    view_size: Point,
    zoom: f32,
}

impl Camera {
    pub fn new(screen_width: f32, screen_height: f32, view_width: f32, view_height: f32) -> Self {
        Self {
            screen_size: Point {
                x: screen_width,
                y: screen_height,
            },
            view_size: Point {
                x: view_width,
                y: view_height,
            },
            view_center: Point { x: 0., y: 0. },
            zoom: 1.,
        }
    }

    pub fn screen_size(&self) -> Point {
        self.screen_size
    }

    pub fn move_by(&mut self, offset: Point) {
        self.view_center.x += offset.x;
        self.view_center.y += offset.y;
    }

    pub fn move_to(&mut self, point: Point) {
        self.view_center = point;
    }

    /// zoom the camera by a factor, e.g. 0.5 zooms out, 2.0 zooms in
    pub fn zoom(&mut self, scale: f32) {
        // FIXME: can return 0 if small enough, maybe just use a zoom scalar
        self.zoom = (self.zoom * scale).max(MIN_ZOOM);
    }

    pub fn world_to_screen_coords(&self, point: Point) -> Point {
        let pixels_per_unit = self.pixels_per_unit();
        let view_offset = Point {
            x: point.x - self.view_center.x,
            y: point.y - self.view_center.y,
        };
        let view_scale = Point {
            x: view_offset.x * pixels_per_unit.x,
            y: view_offset.y * pixels_per_unit.y,
        };
        let x = view_scale.x + self.screen_size.x / 2.0;
        let y = self.screen_size.y - (view_scale.y + self.screen_size.y / 2.0);
        Point { x, y }
    }

    pub fn pixels_per_unit(&self) -> Point {
        Point {
            x: self.screen_size.x / self.view_size.x * self.zoom,
            y: self.screen_size.y / self.view_size.y * self.zoom,
        }
    }
}
