use crate::math::clamp;

pub type Point = ggez::mint::Point2<f32>;
// for back and forth conversion
pub struct MintPoint(Point);

impl MintPoint {
    fn new(x: f32, y: f32) -> Self {
        Self(Point { x, y })
    }
}

impl From<glm::Vec2> for MintPoint {
    fn from(v: glm::Vec2) -> Self {
        MintPoint::new(v.x, v.y) 
    }
}

impl From<Point> for MintPoint {
    fn from(v: Point) -> Self {
        Self(v)
    }
}

impl Into<glm::Vec2> for MintPoint {
    fn into(self) -> glm::Vec2 {
        glm::vec2(self.0.x, self.0.y)
    }
}

impl Into<ggez::mint::Point2<f32>> for MintPoint {
    fn into(self) -> Point {
        self.0
    }
}

pub fn point_distance(p1: &glm::Vec2, p2: &glm::Vec2) -> f32 {
    glm::distance(&p1, &p2)
}

// Angle between 2 points, in radians
pub fn point_angle(p1: &glm::Vec2, p2: &glm::Vec2) -> f32 {
    glm::angle(&p1, &p2)
}

pub fn is_point_in_circle(point: glm::Vec2, circle_pos: glm::Vec2, circle_radius: f32) -> bool {
    (point.x - circle_pos.x).powi(2) + (point.y - circle_pos.y).powi(2) < circle_radius.powi(2)
}

pub fn sign(p1: glm::Vec2, p2: glm::Vec2, p3: glm::Vec2) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

pub fn is_point_in_triangle(point: glm::Vec2, triangle: &[glm::Vec2]) -> bool {
    let d1 = sign(point, triangle[0], triangle[1]);
    let d2 = sign(point, triangle[1], triangle[2]);
    let d3 = sign(point, triangle[2], triangle[0]);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

/// Rotate a point by an angle (in radians) around an origin (clockwise)
pub fn rotate_point(origin: glm::Vec2, point: glm::Vec2, angle: f32) -> glm::Vec2 {
    let cos = angle.cos(); 
    let sin = angle.sin();

    glm::Vec2::new(
        (point.x - origin.x) * cos + (point.y - origin.y) * sin + origin.x,
        (point.y - origin.y) * cos - (point.x - origin.x) * sin + origin.y,
    )
}

pub fn rotate_points(origin: glm::Vec2, points: &[glm::Vec2], angle: f32) -> Vec<glm::Vec2> {
    points
        .iter()
        .map(|p| rotate_point(origin, *p, angle))
        .collect()
}

pub fn heading_to_point(heading: i32) -> glm::Vec2 {
    rotate_point(
        glm::zero(),
        glm::vec2(0.0, 1.0), // north
        (heading as f32).to_radians(),
    )
}

pub fn point_to_heading(p: glm::Vec2) -> i32 {
    let diff = p.x.atan2(p.y).to_degrees() as i32;

    if diff < 0 {
        360 + diff
    } else {
        diff
    }
}

/// https://stackoverflow.com/a/1501725
pub fn distance_line_and_point(line: &[glm::Vec2], p: &glm::Vec2) -> f32 {
    let v = line[0];
    let w = line[1];

    let a = v.y - w.y;
    let b = w.x - v.x;
    let c = v.x * w.y - w.x * v.y;

    (a * p.x + b * p.y + c).abs() / (a * a + b * b).sqrt()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_point_distance() {
        assert_eq!(0., point_distance(&glm::vec2(1., 1.), &glm::vec2(1., 1.)));
        assert_eq!(1., point_distance(&glm::vec2(0., 1.), &glm::vec2(1., 1.)));
        assert_eq!(1., point_distance(&glm::vec2(1., 0.), &glm::vec2(1., 1.)));
    }

    #[test]
    fn test_is_point_in_circle() {
        assert!(is_point_in_circle(glm::zero(), glm::zero(), 1.));
        assert!(is_point_in_circle(glm::vec2(0.5, 0.5), glm::zero(), 1.));
        assert!(!is_point_in_circle(glm::vec2(2., 0.), glm::zero(), 1.));
        assert!(!is_point_in_circle(glm::vec2(0., 2.), glm::zero(), 1.));
        assert!(!is_point_in_circle(glm::vec2(2., 2.), glm::zero(), 1.));
    }

    #[test]
    fn test_is_point_in_triangle() {
        assert!(is_point_in_triangle(glm::zero(), &[
            glm::vec2(-1., -1.),
            glm::vec2(1., 1.),
            glm::vec2(-1., 1.)
        ]));

        assert!(!is_point_in_triangle(glm::vec2(2., 2.), &[
            glm::vec2(-1., -1.),
            glm::vec2(1., 1.),
            glm::vec2(-1., 1.)
        ]));
    }

    #[test]
    fn test_heading_to_point() {
        assert_eq!((0.0, 1.0), (heading_to_point(0).x, heading_to_point(0).y));
        assert_eq!(
            (1.0, 0.0),
            (
                heading_to_point(90).x.trunc(),
                heading_to_point(90).y.trunc()
            )
        );
        assert_eq!(
            (0.0, -1.0),
            (
                heading_to_point(180).x.trunc(),
                heading_to_point(180).y.trunc()
            )
        );
        assert_eq!(
            (-1.0, 0.0),
            (
                heading_to_point(270).x.trunc(),
                heading_to_point(270).y.trunc()
            )
        );
    }

    #[test]
    fn test_point_to_heading() {
        assert_eq!(0, point_to_heading(glm::vec2(0., 1.)));
        assert_eq!(90, point_to_heading(glm::vec2(1., 0.)));
        assert_eq!(180, point_to_heading(glm::vec2(0., -1.)));
        assert_eq!(270, point_to_heading(glm::vec2(-1., 0.)));

        assert_eq!(45, point_to_heading(glm::vec2(1., 1.)));
        assert_eq!(135, point_to_heading(glm::vec2(1., -1.)));
        assert_eq!(225, point_to_heading(glm::vec2(-1., -1.)));
        assert_eq!(315, point_to_heading(glm::vec2(-1., 1.)));
    }

    #[test]
    fn test_distance_line_and_point() {
        assert_eq!(0., distance_line_and_point(&[
            glm::zero(),
            glm::vec2(0., 1.)
        ], &glm::zero()));

        assert_eq!(1., distance_line_and_point(&[
            glm::zero(),
            glm::vec2(0., 1.)
        ], &glm::vec2(1., 0.)));

        assert_eq!(1., distance_line_and_point(&[
            glm::zero(),
            glm::vec2(1., 0.)
        ], &glm::vec2(0., 1.)));
    }
}
