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

pub fn catmull_bezier(points: Vec<Point>) -> Vec<Curve> {
    let mut res = Vec::new();

    let last = points.len() - 1;

    for i in 0..last {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };

        let p1 = points[i];

        let p2 = points[i + 1];

        let p3 = if i + 2 > last {
            points[i + 1]
        } else {
            points[i + 2]
        };

        let c1 = Point {
            x: if i == 0 {
                // this is needed to make the first curve start at x=0
                0.0
            } else {
                (-p0.x + 6.0 * p1.x + p2.x) / 6.0
            },
            y: (-p0.y + 6.0 * p1.y + p2.y) / 6.0,
        };

        let c2 = Point {
            x: ((p1.x + 6.0 * p2.x - p3.x) / 6.0),
            y: ((p1.y + 6.0 * p2.y - p3.y) / 6.0),
        };

        let end = p2;
        res.push(Curve { c1, c2, end });
    }

    res
}
