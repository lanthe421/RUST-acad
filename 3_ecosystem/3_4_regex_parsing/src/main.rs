use once_cell::sync::Lazy;
use regex::Regex;

fn main() {
    println!("parse_nom('>+8.*') = {:?}", parse_nom(">+8.*"));
    println!("parse_regex('>+8.*') = {:?}", parse_regex(">+8.*"))
}

// ── types ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
enum Precision {
    Integer(usize),
    Argument(usize),
    Asterisk,
}

// ── regex impl ───────────────────────────────────────────────────────────────

static REGEX_FORMAT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:.?[<^>])?([+-])?#?0?(\d+)?(?:\.(\d+\$|\*|\d+))?").unwrap()
});


fn parse_nom(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
    nom_parser::parse(input)
}

fn parse_regex(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
    let caps = match REGEX_FORMAT.captures(input) {
        Some(c) => c,
        None => return (None, None, None),
    };

    let sign = caps.get(1).and_then(|m| match m.as_str() {
        "+" => Some(Sign::Plus),
        "-" => Some(Sign::Minus),
        _   => None,
    });

    let width = caps.get(2).and_then(|m| m.as_str().parse().ok());

    let precision = caps.get(3).map(|m| {
        let s = m.as_str();
        if s == "*" {
            Precision::Asterisk
        } else if let Some(n) = s.strip_suffix('$') {
            Precision::Argument(n.parse().unwrap())
        } else {
            Precision::Integer(s.parse().unwrap())
        }
    });

    (sign, width, precision)
}

// ── nom impl ─────────────────────────────────────────────────────────

mod nom_parser {
    use super::{Precision, Sign};
    use nom::{
        branch::alt,
        character::complete::{anychar, char, digit1, one_of},
        combinator::{map, map_res, opt},
        sequence::preceded,
        IResult, Parser,
    };

    fn fill_align(input: &str) -> IResult<&str, ()> {
        let with_fill    = map((anychar, one_of("<^>")), |_| ());
        let without_fill = map(one_of("<^>"), |_| ());
        map(opt(alt((with_fill, without_fill))), |_| ()).parse(input)
    }

    fn sign(input: &str) -> IResult<&str, Option<Sign>> {
        opt(alt((
            map(char('+'), |_| Sign::Plus),
            map(char('-'), |_| Sign::Minus),
        ))).parse(input)
    }

    fn width(input: &str) -> IResult<&str, Option<usize>> {
        opt(map_res(digit1, str::parse)).parse(input)
    }

    fn precision(input: &str) -> IResult<&str, Option<Precision>> {
        opt(preceded(
            char('.'),
            alt((
                map(char('*'), |_| Precision::Asterisk),
                // argument must come before integer to consume digits+'$'
                map(
                    (map_res(digit1, str::parse::<usize>), char('$')),
                    |(n, _)| Precision::Argument(n),
                ),
                map(map_res(digit1, str::parse), Precision::Integer),
            )),
        )).parse(input)
    }

    pub fn parse(input: &str) -> (Option<Sign>, Option<usize>, Option<Precision>) {
        let result: IResult<&str, _> = (
            fill_align,
            sign,
            opt(char('#')),
            opt(char('0')),
            width,
            precision,
        ).parse(input);

        match result {
            Ok((_, (_, s, _, _, w, p))) => (s, w, p),
            Err(_) => (None, None, None),
        }
    }
}


// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod spec {
    use super::*;

    mod nom_impl {
        use super::*;

        #[test]
        fn parses_sign() {
            assert_eq!(parse_nom("").0, None);
            assert_eq!(parse_nom(">8.*").0, None);
            assert_eq!(parse_nom(">+8.*").0, Some(Sign::Plus));
            assert_eq!(parse_nom("-.1$x").0, Some(Sign::Minus));
            assert_eq!(parse_nom("a^#043.8?").0, None);
        }

        #[test]
        fn parses_width() {
            assert_eq!(parse_nom("").1, None);
            assert_eq!(parse_nom(">8.*").1, Some(8));
            assert_eq!(parse_nom(">+8.*").1, Some(8));
            assert_eq!(parse_nom("-.1$x").1, None);
            assert_eq!(parse_nom("a^#043.8?").1, Some(43));
        }

        #[test]
        fn parses_precision() {
            assert_eq!(parse_nom("").2, None);
            assert_eq!(parse_nom(">8.*").2, Some(Precision::Asterisk));
            assert_eq!(parse_nom(">+8.*").2, Some(Precision::Asterisk));
            assert_eq!(parse_nom("-.1$x").2, Some(Precision::Argument(1)));
            assert_eq!(parse_nom("a^#043.8?").2, Some(Precision::Integer(8)));
        }
    }

    mod regex_impl {
        use super::*;

        #[test]
        fn parses_sign() {
            assert_eq!(parse_regex("").0, None);
            assert_eq!(parse_regex(">8.*").0, None);
            assert_eq!(parse_regex(">+8.*").0, Some(Sign::Plus));
            assert_eq!(parse_regex("-.1$x").0, Some(Sign::Minus));
            assert_eq!(parse_regex("a^#043.8?").0, None);
        }

        #[test]
        fn parses_width() {
            assert_eq!(parse_regex("").1, None);
            assert_eq!(parse_regex(">8.*").1, Some(8));
            assert_eq!(parse_regex(">+8.*").1, Some(8));
            assert_eq!(parse_regex("-.1$x").1, None);
            assert_eq!(parse_regex("a^#043.8?").1, Some(43));
        }

        #[test]
        fn parses_precision() {
            assert_eq!(parse_regex("").2, None);
            assert_eq!(parse_regex(">8.*").2, Some(Precision::Asterisk));
            assert_eq!(parse_regex(">+8.*").2, Some(Precision::Asterisk));
            assert_eq!(parse_regex("-.1$x").2, Some(Precision::Argument(1)));
            assert_eq!(parse_regex("a^#043.8?").2, Some(Precision::Integer(8)));
        }
    }
}
