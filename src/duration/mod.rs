use std::fmt;
use std::ops::Deref;

#[derive(PartialEq, Clone, Copy, Debug)]
/// Wrapper type for usize to represent raw seconds before converted into a
/// [`Duration`](struct.Duration.html).
struct RawSeconds(usize);

impl Deref for RawSeconds {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <'a> From<&'a TimeUnit> for RawSeconds {
    /// Convert a [`TimeUnit`](struct.TimeUnit.html) into seconds based on its `kind` and `amount`
    /// fields.
    fn from(t: &'a TimeUnit) -> RawSeconds {
        match t.kind {
            TimeUnitKind::Seconds => RawSeconds(t.amount),
            TimeUnitKind::Minutes => RawSeconds(t.amount * 60),
            TimeUnitKind::Hours => RawSeconds(t.amount * 60 * 60),
            TimeUnitKind::Days => RawSeconds(t.amount * 60 * 60 * 24),
            TimeUnitKind::Years => RawSeconds(t.amount * 60 * 60 * 24 * 365)
        }
    }
}

impl From<Duration> for RawSeconds {
    /// Converts a full [`Duration`](struct.Duration.html) back into seconds.
    fn from(d: Duration) -> RawSeconds {
        RawSeconds(d.iter_units()
            .map(RawSeconds::from)
            .fold(0, |acc, ref n| acc + n.0))
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum TimeUnitKind {
    Seconds = 0,
    Minutes = 1,
    Hours = 2,
    Days = 3,
    Years = 4,
}

#[derive(PartialEq, Clone, Copy)]
pub struct TimeUnit {
    /// The granularity of the amount of time.
    pub kind: TimeUnitKind,
    /// The quantifier for the kind of time unit.
    pub amount: usize,
}

impl TimeUnit {
    fn new(kind: TimeUnitKind, amount: usize) -> Self {
        TimeUnit {
            kind: kind,
            amount: amount,
        }
    }
}

impl fmt::Display for TimeUnit {
    /// Formats `Self` according to: `{amount} {kind}[s if n > 1]`.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut s: String = self.amount.to_string();
        s.push_str(match self.kind {
            TimeUnitKind::Years => " year",
            TimeUnitKind::Days => " day",
            TimeUnitKind::Hours => " hour",
            TimeUnitKind::Minutes => " minute",
            TimeUnitKind::Seconds => " second",
        });

        if self.amount > 1 {
            s.push('s');
        }

        f.write_str(&s)
    }
}

/// Represents parts of a duration with fields of various granularity. Fields are represented by
/// [`TimeUnit`](struct.TimeUnit.html).
#[derive(PartialEq, Clone, Copy)]
pub struct Duration {
    pub seconds: TimeUnit,
    pub minutes: TimeUnit,
    pub hours: TimeUnit,
    pub days: TimeUnit,
    pub years: TimeUnit,
}

impl Duration {
    /// From seconds (in usize), derive a fine-grained [`Duration`](struct.Duration.html).
    pub fn new(seconds: usize) -> Self {
        RawSeconds(seconds).into()
    }

    fn new_zeroed() -> Self {
        Duration {
            seconds: TimeUnit::new(TimeUnitKind::Seconds, 0),
            minutes: TimeUnit::new(TimeUnitKind::Minutes, 0),
            hours: TimeUnit::new(TimeUnitKind::Hours, 0),
            days: TimeUnit::new(TimeUnitKind::Days, 0),
            years: TimeUnit::new(TimeUnitKind::Years, 0),
        }
    }

    fn iter_units(&self) -> impl Iterator<Item = &TimeUnit> {
        vec![
            &self.years,
            &self.days,
            &self.hours,
            &self.minutes,
            &self.seconds,
        ].into_iter()
            .filter(|unit| unit.amount > 0)
    }
}

impl From<RawSeconds> for Duration {
    fn from(mut rs: RawSeconds) -> Duration {
        let mut duration = Duration::new_zeroed();

        duration.years.amount = *rs / (60 * 60 * 24 * 365);
        rs.0 = *rs % (60 * 60 * 24 * 365);

        duration.days.amount = *rs / (60 * 60 * 24);
        rs.0 = *rs % (60 * 60 * 24);

        duration.hours.amount = *rs / (60 * 60);
        rs.0 = *rs % (60 * 60);

        duration.minutes.amount = *rs / (60);
        rs.0 = *rs % (60);

        duration.seconds.amount = rs.0;

        duration
    }
}

impl fmt::Display for Duration {
    /// Rules for formatting:
    /// * ex) 3600 seconds -> "1 hour."
    /// * ex) 3599 seconds -> "59 minutes and 59 seconds."
    /// * ex) 7199 seconds -> "1 hour, 59 minutes and 59 seconds."
    ///     Note) Say there was 1 day additionally to this duration: "1 day, 1 hour, 59 minutes and
    ///     59 seconds.". So, "_x<sub>1</sub>_ _y<sub>1</sub>_, ..., _x<sub>n</sub>_
    ///     _y<sub>n</sub>_, _a_ _b_ and _c_ _d_".
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let units = self.iter_units().collect::<Vec<&TimeUnit>>();

        let s = match units.as_slice() {
            &[] => "".to_string(),
            &[only_unit] => format!("{}.", only_unit),
            &[unit_one, unit_two] => format!("{} and {}.", unit_one, unit_two),
            all_units => {
                let mut s = String::new();
                for unit in all_units.iter().take(all_units.len() - 2) {
                    s.push_str(&format!("{}, ", unit));
                }
                s.push_str(&format!(
                    "{} and {}.",
                    all_units[all_units.len() - 2],
                    all_units[all_units.len() - 1]
                ));
                s
            }
        };
        f.write_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use duration::{Duration, TimeUnit, TimeUnitKind, RawSeconds};

    #[test]
    fn test_partial_eq_timeunit() {
        assert!(
            TimeUnit::new(TimeUnitKind::Seconds, 12) == TimeUnit::new(TimeUnitKind::Seconds, 12)
        );
        assert!(TimeUnit::new(TimeUnitKind::Hours, 5) != TimeUnit::new(TimeUnitKind::Seconds, 12));
        assert!(TimeUnit::new(TimeUnitKind::Hours, 5) != TimeUnit::new(TimeUnitKind::Seconds, 5));
        assert!(TimeUnit::new(TimeUnitKind::Hours, 5) != TimeUnit::new(TimeUnitKind::Hours, 2));
    }

    #[test]
    fn test_display_timeunit() {
        let mut tu_secs = TimeUnit::new(TimeUnitKind::Seconds, 1);
        let mut tu_mins = TimeUnit::new(TimeUnitKind::Minutes, 1);
        let mut tu_hrs = TimeUnit::new(TimeUnitKind::Hours, 1);
        let mut tu_days = TimeUnit::new(TimeUnitKind::Days, 1);
        let mut tu_years = TimeUnit::new(TimeUnitKind::Years, 1);
        assert!(format!("{}", tu_secs) == "1 second");
        assert!(format!("{}", tu_mins) == "1 minute");
        assert!(format!("{}", tu_hrs) == "1 hour");
        assert!(format!("{}", tu_days) == "1 day");
        assert!(format!("{}", tu_years) == "1 year");

        tu_secs = TimeUnit::new(TimeUnitKind::Seconds, 2);
        tu_mins = TimeUnit::new(TimeUnitKind::Minutes, 2);
        tu_hrs = TimeUnit::new(TimeUnitKind::Hours, 2);
        tu_days = TimeUnit::new(TimeUnitKind::Days, 2);
        tu_years = TimeUnit::new(TimeUnitKind::Years, 2);
        assert!(format!("{}", tu_secs) == "2 seconds");
        assert!(format!("{}", tu_mins) == "2 minutes");
        assert!(format!("{}", tu_hrs) == "2 hours");
        assert!(format!("{}", tu_days) == "2 days");
        assert!(format!("{}", tu_years) == "2 years");
    }

    #[test]
    fn test_duration_new() {
        let one_hour = Duration::new(3600);
        assert!(one_hour.seconds == TimeUnit::new(TimeUnitKind::Seconds, 0));
        assert!(one_hour.hours == TimeUnit::new(TimeUnitKind::Hours, 1));
        assert!(one_hour.minutes == TimeUnit::new(TimeUnitKind::Minutes, 0));

        let one_hr_59_min_59_sec = Duration::new(7199);
        assert!(one_hr_59_min_59_sec.seconds.amount == 59);
        assert!(one_hr_59_min_59_sec.minutes.amount == 59);
        assert!(one_hr_59_min_59_sec.hours.amount == 1);
        assert!(one_hr_59_min_59_sec.days.amount == 0);
        assert!(one_hr_59_min_59_sec.years.amount == 0);
    }

    #[test]
    fn test_duration_display() {
        let one_hour = Duration::new(3600);
        assert!(format!("{}", one_hour) == "1 hour.");

        let one_hr_59_min = Duration::new(7140);
        assert!(format!("{}", one_hr_59_min) == "1 hour and 59 minutes.");

        let one_hr_59_min_59_sec = Duration::new(7199);
        assert!(format!("{}", one_hr_59_min_59_sec) == "1 hour, 59 minutes and 59 seconds.");

        let five_units = Duration::new(35_344_799);
        assert!(format!("{}", five_units) == "1 year, 44 days, 1 hour, 59 minutes and 59 seconds.");
    }

    #[test]
    fn test_duration_2_rawsecs() {
        let five_units = Duration::new(35_344_799);
        println!("{:?}", RawSeconds::from(five_units));
        assert!(RawSeconds::from(five_units) == RawSeconds(35_344_799));
    }
}
