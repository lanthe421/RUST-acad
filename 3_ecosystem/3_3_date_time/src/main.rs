use time::{Date, Month, macros::format_description};

fn main() {
    parse_now();
}

const NOW: &str = "2019-06-26";

#[derive(Debug)]
struct User {
    birthdate: Date,
}


fn parse_now() -> Date {
let fmt = format_description!("[year]-[month]-[day]");
    let data = Date::parse(NOW, &fmt).expect("invalid NOW const");
    println!("{data}");
    data
}

impl User {
    fn with_birthdate(year: i32, month: u32, day: u32) -> Self {
        let month = Month::try_from(month as u8).expect("invalid month");
        let birthdate = Date::from_calendar_date(year, month, day as u8)
            .expect("invalid birthdate");
        Self { birthdate }
    }

    /// Returns current age of [`User`] in years.
    fn age(&self) -> u8 {
        let now = parse_now();
        if self.birthdate >= now {
            return 0;
        }

        let years = now.year() - self.birthdate.year();

        if now.ordinal() >= self.birthdate.ordinal() {
            years as u8
        } else {
            (years - 1) as u8
        }
    }

    /// Checks if [`User`] is 18 years old at the moment.
    fn is_adult(&self) -> bool {
        let years = self.age();
        if years >= 18 {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod age_spec {
    use super::*;

    #[test]
    fn counts_age() {
        for ((y, m, d), expected) in vec![
            ((1990, 6, 4), 29),
            ((1990, 7, 4), 28),
            ((0, 1, 1), 0),
            ((1970, 1, 1), 49),
            ((2019, 6, 25), 0),
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), expected);
        }
    }

    #[test]
    fn zero_if_birthdate_in_future() {
        for ((y, m, d), expected) in vec![
            ((2032, 6, 25), 0),
            ((2019, 6, 27), 0), //error in task condition
            ((3000, 6, 27), 0),
            ((9999, 6, 27), 0),
        ] {
            let user = User::with_birthdate(y, m, d);
            assert_eq!(user.age(), expected);
        }
    }

    #[test]
    fn is_adult_when_18_or_older() {
        assert_eq!(User::with_birthdate(2020, 1, 30).is_adult(), false);
        assert_eq!(User::with_birthdate(2421, 12, 24).is_adult(), false);
        assert_eq!(User::with_birthdate(0, 1, 24).is_adult(), false);
    }

    #[test]
    fn birthday_today_counts() {
        let user = User::with_birthdate(2017, 6, 26);
        assert_eq!(user.age(), 2);
    }

    #[test]
    fn if_18_years_today() {
        let user = User::with_birthdate(2001, 6, 26);
        assert_eq!(user.is_adult(), true);
    }

    #[test]
    fn birthday_exactly_today_is_zero() {
        let user = User::with_birthdate(2019, 6, 26);
        assert_eq!(user.age(), 0);
    }

    #[test]
    fn birthday_yesterday_counts_as_one_year() {
        let user = User::with_birthdate(2018, 6, 25);
        assert_eq!(user.age(), 1);
    }

    #[test]
    fn birthday_tomorrow_not_yet() {
        let user = User::with_birthdate(2018, 6, 27);
        assert_eq!(user.age(), 0);
    }

    #[test]
    fn not_adult_day_before_18th_birthday() {
        assert!(!User::with_birthdate(2001, 6, 27).is_adult());
    }

    #[test]
    fn new_year_birthday() {
        let user = User::with_birthdate(2000, 1, 1);
        assert_eq!(user.age(), 19);
    }

    #[test]
    fn dec_31_birthday() {
        let user = User::with_birthdate(2000, 12, 31);
        assert_eq!(user.age(), 18);
    }

}
