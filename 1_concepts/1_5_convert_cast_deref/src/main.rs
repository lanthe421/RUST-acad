use rand::Rng;
use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailString(String);

#[derive(Debug)]
pub struct InvalidEmailError(String);

impl fmt::Display for InvalidEmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid email address: '{}'", self.0)
    }
}

impl EmailString {
    pub fn new(s: impl Into<String>) -> Result<Self, InvalidEmailError> {
        let s = s.into();
        // basic validation: one '@', non-empty local and domain parts, dot in domain
        let valid = s.split('@').collect::<Vec<_>>() == {
            let parts: Vec<&str> = s.splitn(2, '@').collect();
            parts.clone()
        } && {
            let parts: Vec<&str> = s.splitn(2, '@').collect();
            parts.len() == 2
                && !parts[0].is_empty()
                && parts[1].contains('.')
                && !parts[1].starts_with('.')
                && !parts[1].ends_with('.')
        };

        if valid {
            Ok(EmailString(s))
        } else {
            Err(InvalidEmailError(s))
        }
    }
}

impl Deref for EmailString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for EmailString {
    type Error = InvalidEmailError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        EmailString::new(s)
    }
}

impl TryFrom<String> for EmailString {
    type Error = InvalidEmailError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        EmailString::new(s)
    }
}

// Into<String>
impl From<EmailString> for String {
    fn from(e: EmailString) -> String {
        e.0
    }
}

pub struct Random<T> {
    values: [T; 3],
}

impl<T> Random<T> {
    pub fn new(a: T, b: T, c: T) -> Self {
        Random { values: [a, b, c] }
    }

    fn pick(&self) -> &T {
        let i = rand::thread_rng().gen_range(0..3);
        &self.values[i]
    }
}

impl<T> Deref for Random<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.pick()
    }
}

impl<T: fmt::Debug> fmt::Debug for Random<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Random({:?})", self.pick())
    }
}

impl<T: fmt::Display> fmt::Display for Random<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pick())
    }
}

fn main() {
    // EmailString
    let email = EmailString::new("user@example.com").unwrap();
    println!("email = {email}");
    println!("len   = {}", email.len()); // Deref to &str

    let bad = EmailString::new("not-an-email");
    println!("bad   = {:?}", bad);

    // TryFrom
    let e2: EmailString = "hello@world.org".try_into().unwrap();
    let s: String = e2.into(); // From<EmailString> for String
    println!("into String = {s}");

    // Random<i32>
    let r = Random::new(1, 2, 3);
    for _ in 0..5 {
        println!("random i32 = {}", *r);
    }

    // Random<&str> 
    let words = Random::new("apple", "banana", "cherry");
    println!("random word upper = {}", words.to_uppercase()); // Deref -> &str -> .to_uppercase()
}