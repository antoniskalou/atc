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

fn repeat(t: f32, m: f32) -> f32 {
    clamp(t - (t / m).floor() * m, 0.0, m)
}

/// uses radians
fn short_angle_distance(a: f32, b: f32) -> f32 {
    let max = std::f32::consts::PI;
    let da = (b - a) % max;
    2.0 * da % max - da
}

/// return the shortest distance between 2 angles
/// E.g. 350 to 0 will return 10 instead of 350
pub fn angle_lerp(a: f32, b: f32, t: f32) -> f32 {
    let b = if (b - a).abs() > 180.0 {
        b + 360.0
    } else {
        b
    };

    lerp(a, b, t) % 360.0
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub type LerpFn = fn(f32, f32, f32) -> f32;

#[derive(Copy, Clone, Debug)]
pub struct Lerp {
    from: f32,
    to: f32,
    /// total duration in seconds
    duration: f32,
    time: f32,
    lerp_fn: LerpFn,
}

impl Lerp {
    pub fn new(from: f32, to: f32, duration: f32) -> Self {
        Self::with_lerp(from, to, duration, lerp)
    }

    pub fn with_lerp(from: f32, to: f32, duration: f32, lerp_fn: LerpFn) -> Self {
        Self {
            from,
            to,
            duration,
            time: 0.0,
            lerp_fn,
        }
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        let r = (self.lerp_fn)(self.from, self.to, self.time / self.duration);
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
        assert_eq!(180.0, angle_lerp(0.0, 180.0, 1.0));
        // clockwise
        assert_eq!(10.0, angle_lerp(350.0, 10.0, 1.0));
        // anti-clockwise
        assert_eq!(0.0, angle_lerp(10.0, 350.0, 0.5))
    }
}