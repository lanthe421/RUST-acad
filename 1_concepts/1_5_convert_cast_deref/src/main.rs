use derive_more::{Deref, Display, From};
use rand::Rng;
use regex::Regex;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Display, From, Deref)]
#[from(forward)]
#[display("email: {}", _0)]
pub struct EmailString(String);

#[derive(Debug, Display)]
#[display("invalid email address: {}", _0)]
pub struct InvalidEmailError(String);


impl EmailString {
    pub fn new(s: impl Into<String>) -> Result<Self, InvalidEmailError> {
        let s = s.into();
        let email_regex = Regex::new(r"^[^\s@]+@([^\s@]+\.)+[^\s@]+$").unwrap();
        let valid = email_regex.is_match(&s);

        if valid {
            Ok(EmailString(s))
        } else {
            Err(InvalidEmailError(s))
        }
    }
}

impl Into<String> for EmailString {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Debug, Display, Deref)]
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

fn main() {
    // EmailString
    let email = EmailString::new("user@example.com").unwrap();
    println!("{}", email.to_string());
    println!("len = {}", email.len()); // Deref to &str

    let bad = EmailString::new("not-an-email");
    println!("{}", bad.unwrap_err());

    // TryFrom
    let e2: EmailString = "hello@world.org".to_string().try_into().unwrap();
    let s: String = e2.into(); // From<EmailString> for String
    println!("into String = {s}");

    // Random<i32>
    let r = Random::new(1, 2, 3);
    for _ in 0..5 {
        println!("random i32 = {:?}", *r);
    }

    // Random<&str>
    let words = Random::new("apple", "banana", "cherry");
    println!("random word upper = {}", words.pick().to_uppercase()); // Deref -> &str -> .to_uppercase()
}
