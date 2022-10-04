use crate::math::clamp;

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

/// Rotate a point by an angle (in radians) around an origin (clockwise)
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

pub fn heading_to_point(heading: i32) -> Point {
    rotate_point(
        Point { x: 0.0, y: 0.0 },
        Point { x: 0.0, y: 1.0 }, // north
        (heading as f32).to_radians(),
    )
}

pub fn point_to_heading(p: Point) -> i32 {
    let diff = p.x.atan2(p.y).to_degrees() as i32;

    if diff < 0 {
        360 + diff
    } else {
        diff
    }
}

pub fn dot_product(p1: Point, p2: Point) -> f32 {
    p1.x * p2.x + p1.y * p2.y
}

/// https://stackoverflow.com/a/1501725
pub fn distance_line_and_point(line: &[Point; 2], p: Point) -> f32 {
    let [v, w] = *line;
    let length = point_distance(v, w);

    if length == 0. {
        length
    } else {
        let t = clamp(dot_product(
            Point { x: p.x - v.x, y: p.y - v.y },
            Point { x: w.x - v.x, y: w.y - v.y },
        ) / length, 0., 1.);
        let projection = Point {
            x: v.x + t * (w.x - v.x),
            y: v.y + t * (w.y - v.y),
        };
        point_distance(p, projection)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        assert_eq!(0, point_to_heading(Point { x: 0.0, y: 1.0 }));
        assert_eq!(90, point_to_heading(Point { x: 1.0, y: 0.0 }));
        assert_eq!(180, point_to_heading(Point { x: 0.0, y: -1.0 }));
        assert_eq!(270, point_to_heading(Point { x: -1.0, y: 0.0 }));

        assert_eq!(45, point_to_heading(Point { x: 1.0, y: 1.0 }));
        assert_eq!(135, point_to_heading(Point { x: 1.0, y: -1.0 }));
        assert_eq!(225, point_to_heading(Point { x: -1.0, y: -1.0 }));
        assert_eq!(315, point_to_heading(Point { x: -1.0, y: 1.0 }));
    }
}
