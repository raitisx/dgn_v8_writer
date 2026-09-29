//! 2D geometry helpers in UOR space: points, bounds and stored ranges.

use std::f64::consts::{PI, TAU};

/// A 2D point in units of resolution (UOR).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Floating-point bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Point2,
    pub max: Point2,
}

impl Bounds {
    #[must_use]
    pub fn of_point(point: Point2) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    #[must_use]
    pub fn of_points(points: &[Point2]) -> Option<Self> {
        let (first, rest) = points.split_first()?;
        let mut bounds = Self::of_point(*first);
        for point in rest {
            bounds.include(*point);
        }
        Some(bounds)
    }

    pub fn include(&mut self, point: Point2) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }

    #[must_use]
    pub fn union(mut self, other: Self) -> Self {
        self.include(other.min);
        self.include(other.max);
        self
    }
}

/// Integer range exactly as needed for a header: low corner and inclusive
/// high corner in UOR. On disk an element stores `low` followed by
/// `high - low` (FN-E03); a model header stores `low` and `high` (FN-M05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeI64 {
    pub low: [i64; 3],
    pub high: [i64; 3],
}

impl RangeI64 {
    /// Round outward: floor the minimum, ceil the maximum (FN-E03).
    #[must_use]
    pub fn from_bounds(bounds: Bounds) -> Self {
        Self {
            low: [bounds.min.x.floor() as i64, bounds.min.y.floor() as i64, 0],
            high: [bounds.max.x.ceil() as i64, bounds.max.y.ceil() as i64, 0],
        }
    }

    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self {
            low: [
                self.low[0].min(other.low[0]),
                self.low[1].min(other.low[1]),
                self.low[2].min(other.low[2]),
            ],
            high: [
                self.high[0].max(other.high[0]),
                self.high[1].max(other.high[1]),
                self.high[2].max(other.high[2]),
            ],
        }
    }

    #[must_use]
    pub fn extent(self) -> [i64; 3] {
        [
            self.high[0] - self.low[0],
            self.high[1] - self.low[1],
            self.high[2] - self.low[2],
        ]
    }
}

/// Point on an ellipse at parametric angle `t` (FN-E10: angles are
/// parametric, measured from the primary axis).
#[must_use]
pub fn ellipse_point(
    center: Point2,
    primary: f64,
    secondary: f64,
    rotation: f64,
    t: f64,
) -> Point2 {
    let (sin_r, cos_r) = rotation.sin_cos();
    let (sin_t, cos_t) = t.sin_cos();
    Point2::new(
        center.x + primary * cos_t * cos_r - secondary * sin_t * sin_r,
        center.y + primary * cos_t * sin_r + secondary * sin_t * cos_r,
    )
}

/// Tight bounds of a full ellipse.
#[must_use]
pub fn ellipse_bounds(center: Point2, primary: f64, secondary: f64, rotation: f64) -> Bounds {
    let (sin_r, cos_r) = rotation.sin_cos();
    let half_x = ((primary * cos_r).powi(2) + (secondary * sin_r).powi(2)).sqrt();
    let half_y = ((primary * sin_r).powi(2) + (secondary * cos_r).powi(2)).sqrt();
    Bounds {
        min: Point2::new(center.x - half_x, center.y - half_y),
        max: Point2::new(center.x + half_x, center.y + half_y),
    }
}

/// Tight bounds of an elliptical arc from `start` over signed `sweep`.
#[must_use]
pub fn arc_bounds(
    center: Point2,
    primary: f64,
    secondary: f64,
    rotation: f64,
    start: f64,
    sweep: f64,
) -> Bounds {
    if sweep.abs() >= TAU {
        return ellipse_bounds(center, primary, secondary, rotation);
    }
    let point = |t: f64| ellipse_point(center, primary, secondary, rotation, t);
    let mut bounds = Bounds::of_point(point(start));
    bounds.include(point(start + sweep));
    let (sin_r, cos_r) = rotation.sin_cos();
    // dx/dt = 0 and dy/dt = 0 give the axis-extreme parameters.
    let tx = (-secondary * sin_r).atan2(primary * cos_r);
    let ty = (secondary * cos_r).atan2(primary * sin_r);
    for base in [tx, tx + PI, ty, ty + PI] {
        if angle_within_sweep(base, start, sweep) {
            bounds.include(point(base));
        }
    }
    bounds
}

fn angle_within_sweep(angle: f64, start: f64, sweep: f64) -> bool {
    let (from, length) = if sweep >= 0.0 {
        (start, sweep)
    } else {
        (start + sweep, -sweep)
    };
    (angle - from).rem_euclid(TAU) <= length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_rounds_outward() {
        let bounds = Bounds {
            min: Point2::new(-10000.0, 6527.04),
            max: Point2::new(9848.08, 30000.0),
        };
        let range = RangeI64::from_bounds(bounds);
        assert_eq!(range.low, [-10000, 6527, 0]);
        assert_eq!(range.high, [9849, 30000, 0]);
        assert_eq!(range.extent(), [19849, 23473, 0]);
    }

    #[test]
    fn half_circle_bounds() {
        let bounds = arc_bounds(Point2::new(0.0, 0.0), 1.0, 1.0, 0.0, 0.0, PI);
        assert!((bounds.min.x + 1.0).abs() < 1e-12);
        assert!((bounds.max.x - 1.0).abs() < 1e-12);
        assert!(bounds.min.y.abs() < 1e-12);
        assert!((bounds.max.y - 1.0).abs() < 1e-12);
    }

    #[test]
    fn negative_sweep_bounds() {
        // From 180 degrees back to 0 through the top: same as the positive half.
        let bounds = arc_bounds(Point2::new(0.0, 0.0), 1.0, 1.0, 0.0, PI, -PI);
        assert!((bounds.max.y - 1.0).abs() < 1e-12);
        assert!(bounds.min.y.abs() < 1e-12);
    }
}
