use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn to_svg(self) -> String {
        format!("L {} {}", self.x, self.y)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Curve {
    pub c1: Point,
    pub c2: Point,
    pub end: Point,
}

impl Curve {
    pub fn to_svg(&self) -> String {
        format!(
            "C {:.4} {:.4}, {:.4} {:.4}, {:.4} {:.4}",
            self.c1.x, self.c1.y, self.c2.x, self.c2.y, self.end.x, self.end.y
        )
    }
}

/// Convert a list of points to a list of Bézier curves
/// using the Catmull-Rom to Bézier conversion
///
/// # Arguments
///
/// * `points` - A list of points to convert to Bézier curves
///
/// # Returns
///
/// A list of Bézier curves
pub fn catmull_rom_to_bezier(points: Vec<Point>) -> Vec<Curve> {
    if points.len() < 2 {
        return Vec::new();
    }

    let mut curves = Vec::with_capacity(points.len() - 1);

    let last_point = points.len() - 1;
    const TENSION: f64 = 6.0; // Catmull_Rom_standard, adjust to make curves more or less smooth

    for i in 0..last_point {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };

        let p1 = points[i];

        let p2 = points[i + 1];

        let p3 = if i + 2 > last_point {
            points[i + 1]
        } else {
            points[i + 2]
        };

        // Calculate control points using Catmull-Rom to Bézier conversion
        let c1 = Point {
            x: if i == 0 {
                // this is needed to make the first curve start at x=0
                // makes the curve tangent to the x axis for the first point
                0.0
            } else {
                (-p0.x + TENSION * p1.x + p2.x) / TENSION
            },
            y: (-p0.y + TENSION * p1.y + p2.y) / TENSION,
        };

        let c2 = Point {
            x: ((p1.x + TENSION * p2.x - p3.x) / TENSION),
            y: ((p1.y + TENSION * p2.y - p3.y) / TENSION),
        };

        let end = p2;
        curves.push(Curve { c1, c2, end });
    }

    curves
}
