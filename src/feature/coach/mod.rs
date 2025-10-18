use crate::time::TimeSpan;

pub trait Coach {
    fn remaining_time_message(&self, remaining_time: &TimeSpan) -> String;
}

pub struct DefaultItCoach;

impl DefaultItCoach {
    fn remaining_time_component(
        &self,
        component: u64,
        singular: &str,
        plural: &str,
    ) -> Option<String> {
        match component {
            1 => Some(format!("{component} {singular}")),
            n if n > 1 => Some(format!("{component} {plural}")),
            _ => None,
        }
    }
}

impl Coach for DefaultItCoach {
    fn remaining_time_message(&self, remaining_time: &TimeSpan) -> String {
        if remaining_time == &TimeSpan::ZERO {
            "Ora di partire!".to_owned()
        } else {
            let seconds = remaining_time.seconds();
            let minutes = remaining_time.minutes();
            let hours = remaining_time.hours();
            let components = vec![
                self.remaining_time_component(hours, "ora", "ore"),
                self.remaining_time_component(minutes, "minuto", "minuti"),
                self.remaining_time_component(seconds, "secondo", "secondi"),
            ];
            let components: Vec<_> = components.iter().flat_map(|c| c).collect();
            let prefix = if seconds + minutes + hours == 1 {
                "Manca"
            } else {
                "Mancano"
            };
            match components.len() {
                3 => format!(
                    "{prefix} {}, {} e {}",
                    components[0], components[1], components[2]
                ),
                2 => format!("{prefix} {} e {}", components[0], components[1]),
                1 => format!("{prefix} {}", components[0]),
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_message(remaining_time: TimeSpan, expected_message: &str) {
        let message = DefaultItCoach.remaining_time_message(&remaining_time);
        assert_eq!(expected_message, message);
    }

    #[test]
    fn remaining_time_message_should_format_message_it_0s() {
        assert_message(TimeSpan::ZERO, "Ora di partire!");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1s() {
        assert_message(TimeSpan::new(0, 0, 1), "Manca 1 secondo");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_10s() {
        assert_message(TimeSpan::new(0, 0, 10), "Mancano 10 secondi");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1m() {
        assert_message(TimeSpan::new(0, 1, 0), "Manca 1 minuto");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_12m() {
        assert_message(TimeSpan::new(0, 12, 0), "Mancano 12 minuti");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1h() {
        assert_message(TimeSpan::new(1, 0, 0), "Manca 1 ora");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_2h() {
        assert_message(TimeSpan::new(2, 0, 0), "Mancano 2 ore");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1h_12m() {
        assert_message(TimeSpan::new(1, 12, 0), "Mancano 1 ora e 12 minuti");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_5m_30m() {
        assert_message(TimeSpan::new(0, 5, 30), "Mancano 5 minuti e 30 secondi");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1h_20m_30m() {
        assert_message(
            TimeSpan::new(1, 20, 30),
            "Mancano 1 ora, 20 minuti e 30 secondi",
        );
    }
}
