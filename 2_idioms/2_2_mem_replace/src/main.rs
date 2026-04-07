use std::mem;

fn main() {
    let mut s = Solver {
        expected: Trinity { a: 1, b: 2, c: 3 },
        unsolved: vec![
            Trinity { a: 1, b: 2, c: 3 },
            Trinity { a: 2, b: 1, c: 3 },
            Trinity { a: 2, b: 3, c: 1 },
            Trinity { a: 3, b: 1, c: 2 },
        ],
    };
    s.resolve();
    println!("{:?}", s);
}

#[derive(Clone, Debug, PartialEq)]
struct Trinity<T> {
    a: T,
    b: T,
    c: T,
}

impl<T: Clone> Trinity<T> {
    fn rotate(&mut self) {
        let a = self.a.clone();
        let b = self.b.clone();
        let c = self.c.clone();
        self.a = b;
        self.b = c;
        self.c = a;
    }
}

#[derive(Debug)]
struct Solver<T> {
    expected: Trinity<T>,
    unsolved: Vec<Trinity<T>>,
}

impl<T: Clone + PartialEq> Solver<T> {
    fn resolve(&mut self) {
        let unsolved = mem::take(&mut self.unsolved); // забираем вектор, оставляем пустой
        self.unsolved = unsolved
            .into_iter()
            .filter_map(|mut t| {
                for _ in 0..3 {
                    if t == self.expected {
                        return None; // решён — выбрасываем
                    }
                    t.rotate();
                }
                Some(t) // не решён — оставляем
            })
            .collect();
    }

}
