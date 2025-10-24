use error_stack::Report;

use crate::{
    feature::coach::Coach,
    time::{TimeSpan, Timestamp},
};

#[derive(Debug, thiserror::Error)]
#[error("planning error")]
pub struct PlanError;

pub type PlanResult<T> = Result<T, Report<PlanError>>;

#[derive(Debug, PartialEq, Eq)]
pub struct Notification {
    pub time: Timestamp,
    pub message: String,
}

impl Clone for Notification {
    fn clone(&self) -> Self {
        Self {
            time: self.time.clone(),
            message: self.message.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Plan {
    pub rendezvous_time: Timestamp,
    pub trip_duration: TimeSpan,
}

impl Plan {
    pub fn departure_time(&self) -> Timestamp {
        self.rendezvous_time - self.trip_duration
    }

    pub fn notifications<C: Coach>(
        &self,
        now: &Timestamp,
        coach: &C,
    ) -> PlanResult<Vec<Notification>> {
        let departure_time = self.departure_time();

        // Starting from departure time, go in reverse and plan the notifications to be emitted
        // up to now, following the frequency rules.
        let mut time_cursor = departure_time;
        let mut notifications: Vec<Notification> = vec![];
        while &time_cursor >= now {
            let remaining_time = departure_time.time_span_from(&time_cursor);

            // Generate notification for the remaining time
            let notification = Notification {
                time: time_cursor,
                message: coach.remaining_time_message(&remaining_time),
            };
            notifications.push(notification);

            // Go back for the next (backward in time) notification to generate accoding to the
            // remaining time (relative to the cursor).
            let cursor_back_span = if remaining_time < TimeSpan::of_minutes(5) {
                TimeSpan::of_minutes(1)
            } else if remaining_time < TimeSpan::of_minutes(30) {
                TimeSpan::of_minutes(5)
            } else if remaining_time < TimeSpan::of_hours(1) {
                TimeSpan::of_minutes(10)
            } else {
                TimeSpan::of_minutes(15)
            };
            time_cursor = time_cursor - cursor_back_span;
        }
        Ok(notifications)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCoach;
    impl Coach for TestCoach {
        fn remaining_time_message(&self, remaining_time: &TimeSpan) -> String {
            format!("remaining: {:?}", remaining_time)
        }
    }

    fn notification_go(rendezvous_time: Timestamp) -> Notification {
        notification_from(rendezvous_time, TimeSpan::ZERO)
    }

    fn notification_from(rendezvous_time: Timestamp, time_span: TimeSpan) -> Notification {
        Notification {
            time: rendezvous_time - time_span,
            message: TestCoach.remaining_time_message(&time_span),
        }
    }

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

    #[test]
    fn notifications_for_past_departure() {
        let now = Timestamp::now().unwrap();
        let rendezvous_time = now - TimeSpan::of_minutes(5);
        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let notifications = plan.notifications(&now, &TestCoach).unwrap();

        let expected: Vec<Notification> = vec![];
        assert_eq!(expected, notifications);
    }

    #[test]
    fn notifications_for_immediate_departure() {
        let rendezvous_time = Timestamp::now().unwrap();
        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let notifications = plan.notifications(&rendezvous_time, &TestCoach).unwrap();

        let expected: Vec<Notification> = vec![notification_go(rendezvous_time)];
        assert_eq!(expected, notifications);
    }

    #[test]
    fn notifications_for_last_5m_every_1m() {
        let now = Timestamp::now().unwrap();
        let rendezvous_time = now + TimeSpan::of_minutes(5);
        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let notifications = plan.notifications(&now, &TestCoach).unwrap();

        let expected: Vec<Notification> = vec![
            notification_go(rendezvous_time),
            notification_from(rendezvous_time, TimeSpan::of_minutes(1)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(2)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(3)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(4)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(5)),
        ];
        assert_eq!(expected, notifications);
    }

    #[test]
    fn notifications_from_last_5m_to_30m_every_5m() {
        let now = Timestamp::now().unwrap();
        let rendezvous_time = now + TimeSpan::of_minutes(30);
        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let notifications = plan.notifications(&now, &TestCoach).unwrap();
        let filtered: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.time < (rendezvous_time - TimeSpan::of_minutes(5)))
            .collect();

        let expected: Vec<Notification> = vec![
            notification_from(rendezvous_time, TimeSpan::of_minutes(10)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(15)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(20)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(25)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(30)),
        ];
        assert_eq!(expected, filtered);
    }

    #[test]
    fn notifications_from_last_30m_to_1h_every_10m() {
        let now = Timestamp::now().unwrap();
        let rendezvous_time = now + TimeSpan::of_hours(1);
        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let notifications = plan.notifications(&now, &TestCoach).unwrap();
        let filtered: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.time < (rendezvous_time - TimeSpan::of_minutes(30)))
            .collect();

        let expected: Vec<Notification> = vec![
            notification_from(rendezvous_time, TimeSpan::of_minutes(40)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(50)),
            notification_from(rendezvous_time, TimeSpan::of_minutes(60)),
        ];
        assert_eq!(expected, filtered);
    }

    #[test]
    fn notifications_from_last_1h_onward_every_15m() {
        let now = Timestamp::now().unwrap();
        let rendezvous_time = now + TimeSpan::of_hours(3);
        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let notifications = plan.notifications(&now, &TestCoach).unwrap();
        let filtered: Vec<_> = notifications
            .into_iter()
            .filter(|n| n.time < (rendezvous_time - TimeSpan::of_hours(1)))
            .collect();

        let expected: Vec<Notification> = vec![
            notification_from(rendezvous_time, TimeSpan::new(1, 15, 0)),
            notification_from(rendezvous_time, TimeSpan::new(1, 30, 0)),
            notification_from(rendezvous_time, TimeSpan::new(1, 45, 0)),
            notification_from(rendezvous_time, TimeSpan::new(2, 0, 0)),
            notification_from(rendezvous_time, TimeSpan::new(2, 15, 0)),
            notification_from(rendezvous_time, TimeSpan::new(2, 30, 0)),
            notification_from(rendezvous_time, TimeSpan::new(2, 45, 0)),
            notification_from(rendezvous_time, TimeSpan::new(3, 0, 0)),
        ];
        assert_eq!(expected, filtered);
    }
}
