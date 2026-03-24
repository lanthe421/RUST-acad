#[derive(Copy, Clone, Debug, Default)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Clone, Debug)]
struct Polyline {
    points: Vec<Point>,
}

impl Polyline {
    fn new(first: Point) -> Self {
        Polyline { points: vec![first] }
    }
}

fn main() {
    let point1 = Point { x: 5.0, y: 7.3 };
    let point2 = Point { x: 3.0, y: 4.3 };

    let line1 = Polyline::new(point1);
    let mut line2 = line1.clone();
    line2.points.push(point2);

    println!("{:?}", line1);
    println!("{:?}", line2);
}
