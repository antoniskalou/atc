// Alternative to val.clamp(..) because it doesn't handle negative
// values correctly
pub fn clamp<N>(val: N, min: N, max: N) -> N
where
    N: std::cmp::PartialOrd,
{
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

/// Returns the sign of the value. -1 or 1, or 0 if value is zero
pub fn sign(s: f32) -> f32 {
    if s < 0.0 {
        -1.0
    } else if s > 0.0 {
        1.0
    } else {
        0.0
    }
}

/// Returns the inverse of the angle, in radians.
pub fn complement_angle(angle: f32) -> f32 {
    angle - 360.0 * sign(angle)
}

pub fn long_angle_distance(a: f32, b: f32) -> f32 {
    complement_angle(short_angle_distance(a, b))
}

/// Returns the shortest angle distance in degrees.
///
/// Positive values represent a right direction, while negative values
/// represent a left direction.
///
/// See https://stackoverflow.com/a/28037434
pub fn short_angle_distance(a: f32, b: f32) -> f32 {
    (b - a + 180.0).rem_euclid(360.0) - 180.0
}

/// return the shortest distance between 2 angles
/// E.g. 350 to 0 will return 10 instead of 350
///
/// See https://gist.github.com/shaunlebron/8832585?permalink_comment_id=3227412#gistcomment-3227412
pub fn angle_lerp(a: f32, b: f32, t: f32) -> f32 {
    (a + short_angle_distance(a, b) * t).rem_euclid(360.0)
}

pub fn long_angle_lerp(a: f32, b: f32, t: f32) -> f32 {
    (a + long_angle_distance(a, b) * t).rem_euclid(360.0)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub type InterpolatorFn = fn(f32, f32, f32) -> f32;

#[derive(Copy, Clone, Debug)]
pub struct Interpolator {
    from: f32,
    to: f32,
    /// total duration in seconds
    duration: f32,
    time: f32,
    i_fn: InterpolatorFn,
}

impl Interpolator {
    pub fn new(from: f32, to: f32, duration: f32) -> Self {
        Self::with_fn(from, to, duration, lerp)
    }

    pub fn with_fn(from: f32, to: f32, duration: f32, i_fn: InterpolatorFn) -> Self {
        Self {
            from,
            to,
            duration,
            time: 0.0,
            i_fn,
        }
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        let r = (self.i_fn)(self.from, self.to, self.time / self.duration);
        self.time += dt;
        r
    }

    pub fn is_finished(&self) -> bool {
        self.time >= self.duration
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sign() {
        assert_eq!(-1.0, sign(-6.0));
        assert_eq!(1.0, sign(10.0));
        assert_eq!(0.0, sign(0.0));
    }

    #[test]
    fn test_complement_angle() {
        assert_eq!(0.0, complement_angle(0.0));
        assert_eq!(-270.0, complement_angle(90.0));
        assert_eq!(-180.0, complement_angle(180.0));
    }

    #[test]
    fn test_long_angle_distance() {
        assert_eq!(-270.0, long_angle_distance(0.0, 90.0));
        assert_eq!(180.0, long_angle_distance(0.0, 180.0));
        assert_eq!(-340.0, long_angle_distance(350.0, 10.0).round());
        assert_eq!(180.0, long_angle_distance(90.0, 270.0));
        assert_eq!(180.0, long_angle_distance(270.0, 90.0));
    }

    #[test]
    fn test_short_angle_distance() {
        assert_eq!(20.0, short_angle_distance(350.0, 10.0));
        assert_eq!(-20.0, short_angle_distance(10.0, 350.0));
        assert_eq!(-180.0, short_angle_distance(90.0, 270.0));
        assert_eq!(-180.0, short_angle_distance(270.0, 90.0));
    }

    #[test]
    fn test_clamp() {
        // i32
        assert_eq!(0, clamp(-1, 0, 1));
        assert_eq!(1, clamp(2, 0, 1));
        assert_eq!(1, clamp(1, 0, 2));

        // f32
        assert_eq!(0.0, clamp(-1.0, 0.0, 1.0));
        assert_eq!(1.0, clamp(2.0, 0.0, 1.0));
        assert_eq!(1.0, clamp(1.0, 0.0, 2.0));
    }

    #[test]
    fn test_angle_lerp() {
        assert_eq!(10.0, angle_lerp(350.0, 10.0, 1.0));
        assert_eq!(0.0, angle_lerp(350.0, 10.0, 0.5));

        assert_eq!(0.0, angle_lerp(90.0, 0.0, 1.0));
        assert_eq!(45.0, angle_lerp(90.0, 0.0, 0.5));

        assert_eq!(350.0, angle_lerp(10.0, 350.0, 1.0));
        assert_eq!(0.0, angle_lerp(10.0, 350.0, 0.5));
    }

    #[test]
    fn test_long_angle_lerp() {
        assert_eq!(10.0, long_angle_lerp(350.0, 10.0, 1.0));
        assert_eq!(180.0, long_angle_lerp(350.0, 10.0, 0.5));

        assert_eq!(0.0, long_angle_lerp(90.0, 0.0, 1.0));
        assert_eq!(225.0, long_angle_lerp(90.0, 0.0, 0.5));

        assert_eq!(350.0, long_angle_lerp(10.0, 350.0, 1.0));
        assert_eq!(180.0, long_angle_lerp(10.0, 350.0, 0.5));
    }
}
