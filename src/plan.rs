use crate::time::{TimeSpan, Timestamp};

#[derive(Debug)]
struct Notification {
    time: Timestamp,
    message: String
}

#[derive(Debug)]
pub struct Plan {
    pub rendezvous_time: Timestamp,
    pub trip_duration: TimeSpan,
}

impl Plan {
    pub fn departure_time(&self) -> Timestamp {
        &self.rendezvous_time - &self.trip_duration
    }
    
    fn notifications(&self) -> Vec<Notification> {
        todo!()
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

    // #[test]
    // fn notifications_for_past_departure() {
    //     let now = Timestamp::now().unwrap();
    //     let rendezvous_time = now - TimeSpan::of_minutes(5);
    //     let plan = Plan {
    //         rendezvous_time,
    //         trip_duration: TimeSpan::ZERO
    //     };
        
    //     let notifications = plan.notifications();

    //     let expected = vec![];
    //     assert_eq!(expected, notifications);
    // }
}
