use std::{cmp::Ordering, env, io};

fn main() {
    println!("Guess the number!");

    let secret_number = get_secret_number();

    loop {
        println!("Please input your guess.");

        let guess = match get_guess_number() {
            Some(n) => n,
            _ => continue,
        };

        println!("You guessed: {}", guess);

        match check_guess(guess, secret_number) {
            GuessResult::TooSmall => println!("Too small!"),
            GuessResult::TooBig => println!("Too big!"),
            GuessResult::Correct => {
                println!("You win!");
                break;
            }
        }
    }
}

/// Result of comparing a guess to the secret number.
#[derive(Debug, PartialEq)]
pub enum GuessResult {
    TooSmall,
    TooBig,
    Correct,
}

/// Compare a guess against the secret number.
pub fn check_guess(guess: u32, secret: u32) -> GuessResult {
    match guess.cmp(&secret) {
        Ordering::Less => GuessResult::TooSmall,
        Ordering::Greater => GuessResult::TooBig,
        Ordering::Equal => GuessResult::Correct,
    }
}

/// Parse a secret number from a string slice.
pub fn parse_secret(s: &str) -> Option<u32> {
    s.trim().parse().ok()
}

/// Parse a guess from a string slice.
pub fn parse_guess(s: &str) -> Option<u32> {
    s.trim().parse().ok()
}

fn get_secret_number() -> u32 {
    let secret_number = env::args()
        .skip(1)
        .take(1)
        .last()
        .expect("No secret number is specified");
    parse_secret(&secret_number).expect("Secret number is not a number")
}

fn get_guess_number() -> Option<u32> {
    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("Failed to read line");
    parse_guess(&guess)
}

#[cfg(test)]
mod guess_result_spec {
    use super::*;

    #[test]
    fn returns_too_small_when_guess_is_lower() {
        assert_eq!(check_guess(3, 10), GuessResult::TooSmall);
    }

    #[test]
    fn returns_too_big_when_guess_is_higher() {
        assert_eq!(check_guess(15, 10), GuessResult::TooBig);
    }

    #[test]
    fn returns_correct_when_guess_matches_secret() {
        assert_eq!(check_guess(10, 10), GuessResult::Correct);
    }

    #[test]
    fn returns_correct_for_zero() {
        assert_eq!(check_guess(0, 0), GuessResult::Correct);
    }

    #[test]
    fn returns_too_small_for_zero_vs_one() {
        assert_eq!(check_guess(0, 1), GuessResult::TooSmall);
    }

    #[test]
    fn returns_too_big_for_max_u32() {
        assert_eq!(check_guess(u32::MAX, u32::MAX - 1), GuessResult::TooBig);
    }
}

#[cfg(test)]
mod parse_secret_spec {
    use super::*;

    #[test]
    fn parses_valid_number() {
        assert_eq!(parse_secret("42"), Some(42));
    }

    #[test]
    fn parses_number_with_whitespace() {
        assert_eq!(parse_secret("  7  "), Some(7));
    }

    #[test]
    fn returns_none_for_empty_string() {
        assert_eq!(parse_secret(""), None);
    }

    #[test]
    fn returns_none_for_non_numeric_input() {
        assert_eq!(parse_secret("abc"), None);
    }

    #[test]
    fn returns_none_for_negative_number() {
        assert_eq!(parse_secret("-5"), None);
    }

    #[test]
    fn returns_none_for_float() {
        assert_eq!(parse_secret("3.14"), None);
    }

    #[test]
    fn parses_zero() {
        assert_eq!(parse_secret("0"), Some(0));
    }

    #[test]
    fn parses_max_u32() {
        assert_eq!(parse_secret("4294967295"), Some(u32::MAX));
    }

    #[test]
    fn returns_none_for_overflow() {
        assert_eq!(parse_secret("4294967296"), None);
    }
}

#[cfg(test)]
mod parse_guess_spec {
    use super::*;

    #[test]
    fn parses_valid_guess() {
        assert_eq!(parse_guess("100"), Some(100));
    }

    #[test]
    fn parses_guess_with_newline() {
        assert_eq!(parse_guess("55\n"), Some(55));
    }

    #[test]
    fn returns_none_for_empty_input() {
        assert_eq!(parse_guess(""), None);
    }

    #[test]
    fn returns_none_for_letters() {
        assert_eq!(parse_guess("hello"), None);
    }

    #[test]
    fn returns_none_for_whitespace_only() {
        assert_eq!(parse_guess("   "), None);
    }
}

#[cfg(test)]
mod game_flow_spec {
    use super::*;

    #[test]
    fn correct_guess_on_first_try() {
        let secret = 42;
        let guesses = [42];
        let mut result = GuessResult::TooSmall;
        for g in guesses {
            result = check_guess(g, secret);
            if result == GuessResult::Correct {
                break;
            }
        }
        assert_eq!(result, GuessResult::Correct);
    }

    #[test]
    fn binary_search_converges_to_correct() {
        let secret = 73u32;
        let mut lo = 0u32;
        let mut hi = 100u32;
        let mut found = false;

        while lo <= hi {
            let mid = lo + (hi - lo) / 2;
            match check_guess(mid, secret) {
                GuessResult::Correct => {
                    found = true;
                    break;
                }
                GuessResult::TooSmall => lo = mid + 1,
                GuessResult::TooBig => hi = mid - 1,
            }
        }

        assert!(found);
    }

    #[test]
    fn invalid_guesses_are_skipped() {
        let inputs = ["abc", "3.14", "", "  "];
        for input in inputs {
            assert_eq!(parse_guess(input), None);
        }
    }
}
