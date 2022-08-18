// alternative to val.clamp(..) because it doesn't handle negative
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

/// See https://stackoverflow.com/a/28037434
pub fn short_angle_distance(a: f32, b: f32) -> f32 {
    let diff = (b - a + 180.0) % 360.0 - 180.0;
    if diff < -180.0 { diff + 360.0 } else { diff }
}

fn repeat(t: f32, m: f32) -> f32 {
    clamp(t - (t / m).floor() * m, 0.0, m)
}

/// return the shortest distance between 2 angles
/// E.g. 350 to 0 will return 10 instead of 350
///
/// See https://gist.github.com/shaunlebron/8832585?permalink_comment_id=3227412#gistcomment-3227412
pub fn angle_lerp(a: f32, b: f32, t: f32) -> f32 {
    let dt = repeat(b - a, 360.0);
    let lerp = lerp(a, a + if dt > 180.0 { dt - 360.0 } else { dt }, t) % 360.0;

    if lerp < 0.0 {
        360.0 + lerp
    } else {
        lerp
    }
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
    fn test_short_angle_distance() {
        assert_eq!(20.0, short_angle_distance(350.0, 10.0));
        assert_eq!(20.0, short_angle_distance(10.0, 350.0));
        assert_eq!(180.0, short_angle_distance(90.0, 270.0));
        assert_eq!(180.0, short_angle_distance(270.0, 90.0));
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
}
