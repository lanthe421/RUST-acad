use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Fact<T: ?Sized> {
    _marker: PhantomData<T>,
}

pub trait Facts {
    fn facts() -> &'static [&'static str];
}

impl<T> Fact<T>
where
    T: ?Sized + Facts,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn fact(&self) -> &'static str {
        let facts = T::facts();
        // Simple pseudo-random index using system time nanoseconds
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;
        facts[nanos % facts.len()]
    }
}

impl<T> Facts for Vec<T> {
    fn facts() -> &'static [&'static str] {
        &[
            "Vec is heap-allocated.",
            "Vec may re-allocate on growing.",
            "Vec has O(1) amortized push.",
            "Vec stores elements contiguously in memory.",
            "Vec capacity can be reserved upfront to avoid re-allocations.",
        ]
    }
}

impl Facts for str {
    fn facts() -> &'static [&'static str] {
        &[
            "str is always valid UTF-8.",
            "str is an unsized type, usually seen as &str.",
            "str slices are immutable views into string data.",
        ]
    }
}

impl Facts for i32 {
    fn facts() -> &'static [&'static str] {
        &[
            "i32 is a signed 32-bit integer.",
            "i32 ranges from -2,147,483,648 to 2,147,483,647.",
            "i32 is the default integer type in Rust.",
        ]
    }
}

fn main() {
    let f: Fact<Vec<i32>> = Fact::new();
    println!("Fact about Vec: {}", f.fact());
    println!("Fact about Vec: {}", f.fact());

    let f2: Fact<str> = Fact::new();
    println!("Fact about str: {}", f2.fact());

    let f3: Fact<i32> = Fact::new();
    println!("Fact about i32: {}", f3.fact());
}
