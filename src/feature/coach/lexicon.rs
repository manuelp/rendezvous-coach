use chrono::TimeDelta;

use crate::time::*;

fn remaining_time_component(component: i64, singular: &str, plural: &str) -> Option<String> {
    match component {
        1 => Some(format!("{component} {singular}")),
        n if n > 1 => Some(format!("{component} {plural}")),
        _ => None,
    }
}

pub fn remaining_time_message(remaining_time: &TimeDelta) -> String {
    let seconds = time_delta_seconds(remaining_time);
    let minutes = time_delta_minutes(remaining_time);
    let hours = time_delta_hours(remaining_time);
    let components = vec![
        remaining_time_component(hours, "ora", "ore"),
        remaining_time_component(minutes, "minuto", "minuti"),
        remaining_time_component(seconds, "secondo", "secondi"),
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

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    fn assert_message(remaining_time: TimeDelta, expected_message: &str) {
        let message = remaining_time_message(&remaining_time);
        assert_eq!(expected_message, message);
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1s() {
        assert_message(TimeDelta::seconds(1), "Manca 1 secondo");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_10s() {
        assert_message(TimeDelta::seconds(10), "Mancano 10 secondi");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1m() {
        assert_message(TimeDelta::minutes(1), "Manca 1 minuto");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_12m() {
        assert_message(TimeDelta::minutes(12), "Mancano 12 minuti");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1h() {
        assert_message(TimeDelta::hours(1), "Manca 1 ora");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_2h() {
        assert_message(TimeDelta::hours(2), "Mancano 2 ore");
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1h_12m() {
        assert_message(
            TimeDelta::hours(1) + TimeDelta::minutes(12),
            "Mancano 1 ora e 12 minuti",
        );
    }

    #[test]
    fn remaining_time_message_should_format_message_it_5m_30m() {
        assert_message(
            TimeDelta::minutes(5) + TimeDelta::seconds(30),
            "Mancano 5 minuti e 30 secondi",
        );
    }

    #[test]
    fn remaining_time_message_should_format_message_it_1h_20m_30m() {
        assert_message(
            TimeDelta::hours(1) + TimeDelta::minutes(20) + TimeDelta::seconds(30),
            "Mancano 1 ora, 20 minuti e 30 secondi",
        );
    }
}
