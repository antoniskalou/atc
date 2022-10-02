use crate::geom::Point;

#[derive(Clone, Debug)]
pub struct Camera {
    screen_size: Point,
    view_center: Point,
}

impl Camera {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let screen_size = Point { x: screen_width, y: screen_height };
        Self {
            screen_size,
            view_center: Point { x: 0., y: 0. },
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

    pub fn world_to_screen_coords(
        &self,
        point: Point,
        scale: f32,
    ) -> Point {
        let x = point.x * scale + self.screen_size.x / 2.;
        let y = self.screen_size.y - (point.y * scale + self.screen_size.y / 2.);
        Point {
            x: x - self.view_center.x,
            y: y + self.view_center.y,
        }
    }
}