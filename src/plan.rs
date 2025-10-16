use chrono::{prelude::*, TimeDelta};

use crate::time::TimeSpan;

#[derive(Debug)]
pub struct Plan {
    pub rendezvous_time: DateTime<Local>,
    pub trip_duration: TimeSpan,
}

impl Plan {
    pub fn departure_time(&self) -> DateTime<Local> {
        self.rendezvous_time - TimeDelta::from(&self.trip_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn departure_time() {
        let plan = Plan {
            rendezvous_time: Local.with_ymd_and_hms(2025, 10, 15, 13, 00, 00).unwrap(),
            trip_duration: TimeSpan::new(0, 20, 0),
        };

        let departure_time = plan.departure_time();

        assert_eq!(
            Local.with_ymd_and_hms(2025, 10, 15, 12, 40, 00).unwrap(),
            departure_time
        );
    }
}
