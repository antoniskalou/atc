pub type Point = ggez::mint::Point2<f32>;

pub fn point_distance(p1: Point, p2: Point) -> f32 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}

pub fn is_point_in_circle(point: Point, circle_pos: Point, circle_radius: f32) -> bool {
    (point.x - circle_pos.x).powi(2) + (point.y - circle_pos.y).powi(2) < circle_radius.powi(2)
}

pub fn sign(p1: Point, p2: Point, p3: Point) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

pub fn is_point_in_triangle(point: Point, triangle: Vec<Point>) -> bool {
    let d1 = sign(point, triangle[0], triangle[1]);
    let d2 = sign(point, triangle[1], triangle[2]);
    let d3 = sign(point, triangle[2], triangle[0]);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

/// Rotate a point by an angle (in degrees) around an origin (clockwise)
pub fn rotate_point(origin: Point, point: Point, angle: f32) -> Point {
    let cos = angle.cos();
    let sin = angle.sin();

    Point {
        x: (point.x - origin.x) * cos + (point.y - origin.y) * sin + origin.x,
        y: (point.y - origin.y) * cos - (point.x - origin.x) * sin + origin.y,
    }
}

pub fn rotate_points(origin: Point, points: &[Point], angle: f32) -> Vec<Point> {
    points
        .iter()
        .map(|p| rotate_point(origin, *p, angle))
        .collect()
}

pub fn heading_to_vector(heading: i32) -> Point {
    rotate_point(
        Point { x: 0.0, y: 0.0 },
        Point { x: 0.0, y: 1.0 }, // north
        (heading as f32).to_radians(),
    )
}

// pub fn vector_to_heading(p: Point) -> i32 {
//     p.x.atan2(p.y).trunc() as i32
// }

/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
/// pointing downward and the origin at the top-left,
pub fn world_to_screen_coords(screen_width: f32, screen_height: f32, point: Point) -> Point {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
    Point { x, y }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_heading_to_vector() {
        assert_eq!((0.0, 1.0), (heading_to_vector(0).x, heading_to_vector(0).y));
        assert_eq!(
            (1.0, 0.0),
            (
                heading_to_vector(90).x.trunc(),
                heading_to_vector(90).y.trunc()
            )
        );
        assert_eq!(
            (0.0, -1.0),
            (
                heading_to_vector(180).x.trunc(),
                heading_to_vector(180).y.trunc()
            )
        );
        assert_eq!(
            (-1.0, 0.0),
            (
                heading_to_vector(270).x.trunc(),
                heading_to_vector(270).y.trunc()
            )
        );
    }
}
