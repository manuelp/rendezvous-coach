use crate::time::{TimeSpan, Timestamp};

#[derive(Debug)]
pub struct Plan {
    pub rendezvous_time: Timestamp,
    pub trip_duration: TimeSpan,
}

impl Plan {
    pub fn departure_time(&self) -> Timestamp {
        &self.rendezvous_time - &self.trip_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn departure_time() {
        let plan = Plan {
            rendezvous_time: Timestamp::new(2025, 10, 15, 13, 00, 00).unwrap(),
            trip_duration: TimeSpan::new(0, 20, 0),
        };

        assert_eq!(
            Timestamp::new(2025, 10, 15, 12, 40, 00).unwrap(),
            plan.departure_time()
        );
    }
}
