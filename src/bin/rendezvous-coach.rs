use std::collections::VecDeque;
use std::time::Duration;

use clap::Parser;
use error_stack::ResultExt;
use ratatui::{
    Frame, Terminal, TerminalOptions, Viewport,
    backend::Backend,
    buffer::Buffer,
    crossterm::event,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, LineGauge, List, ListItem, Widget},
};
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::feature::coach::{Coach, DefaultItCoach};
use rendezvous_coach::feature::tts::{Speaker, TTSSpeaker};
use rendezvous_coach::init;
use rendezvous_coach::plan::{Notification, Plan};
use rendezvous_coach::time::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Rendezvous time
    #[arg(short, long, value_name = "HH:MM")]
    rendezvous: String,
    /// Trip duration
    #[arg(short, long, value_name = "HH:MM")]
    trip: String,
}

#[derive(Debug)]
struct Notifications {
    pending: Vec<Notification>,
    emitted: VecDeque<Notification>,
    max_emitted: usize,
}

impl Notifications {
    fn new(pending: Vec<Notification>, max_emitted: usize) -> Self {
        Self {
            pending,
            emitted: VecDeque::with_capacity(max_emitted),
            max_emitted,
        }
    }

    fn emit(&mut self, n: Notification) {
        self.emitted.push_front(n);
        self.emitted.truncate(self.max_emitted);
    }
}

#[derive(Debug)]
struct AppState {
    departure_time: Timestamp,
    started: Timestamp,
    notifications: Notifications,
    exit: bool,
}

impl AppState {
    fn new<C: Coach>(plan: &Plan, coach: C, max_messages: usize) -> AppResult<Self> {
        let now = Timestamp::now().change_context(AppError)?;
        let pending = plan.notifications(&now, &coach).change_context(AppError)?;
        let notifications = Notifications::new(pending, max_messages);
        Ok(Self {
            departure_time: plan.departure_time(),
            started: Timestamp::now().change_context(AppError)?,
            notifications,
            exit: false,
        })
    }

    fn total_time(&self) -> TimeSpan {
        self.departure_time.time_span_from(&self.started)
    }

    // fn elapsed_time(&self, now: &Timestamp) -> TimeSpan {
    //     now.time_span_from(&self.started)
    // }

    fn remaining_time(&self, now: &Timestamp) -> TimeSpan {
        self.departure_time.time_span_from(now)
    }

    fn remaining_ratio(&self, now: &Timestamp) -> f64 {
        let total_secs = self.total_time().total_secs() as f64;
        let remaing_secs = self.remaining_time(now).total_secs() as f64;
        remaing_secs / total_secs
    }

    fn tick<S: Speaker>(&mut self, speaker: &mut S) -> AppResult<Timestamp> {
        let now = Timestamp::now().change_context(AppError)?;
        if self.notifications.pending.is_empty() {
            self.exit = true;
        } else {
            if let Some(n) = self.notifications.pending.pop_if(|n| n.time == now) {
                self.notifications.emit(n.clone());
                speaker.speak(&n.message).change_context(AppError)?;

                if let Some(next_notification) = self.notifications.pending.last() {
                    let to_next = next_notification.time.time_span_from(&now);
                    let msg = format!("Prossima notifica tra: {}", to_next);
                    self.notifications.emit(Notification { message: msg, ..n });
                }
            }
        }
        Ok(now)
    }

    fn run<B: Backend, S: Speaker>(
        &mut self,
        terminal: &mut Terminal<B>,
        speaker: &mut S,
    ) -> AppResult<()> {
        let tick_time = Duration::from_secs(1);
        loop {
            let now = self.tick(speaker)?;

            terminal
                .draw(|frame| self.draw(frame))
                .change_context(AppError)
                .attach("cannot render frame")?;

            self.handle_events(tick_time)?;

            if self.remaining_time(&now) == TimeSpan::ZERO {
                self.exit = true;
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, poll_time: Duration) -> AppResult<()> {
        let event_available = event::poll(poll_time)
            .change_context(AppError)
            .attach("cannot read event")?;
        if event_available {
            match event::read()
                .change_context(AppError)
                .attach("cannot read event")?
            {
                event::Event::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
                    match key_event.code {
                        event::KeyCode::Char('q') => self.exit = true,
                        _ => (),
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}

impl Widget for &AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(vec![
            Span::styled(
                "Departure time",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" 🚗 "),
            Span::styled(
                format!("{}", self.departure_time),
                Style::default().fg(Color::Green),
            )
            .add_modifier(Modifier::ITALIC),
            Span::raw(" | (q) Quit"),
        ]);
        let block = Block::new().title(title.centered());
        block.render(area, buf);

        let vertical = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(self.notifications.max_emitted as u16),
        ])
        .margin(1);
        let [progress_area, main] = vertical.areas(area);

        let now = Timestamp::now().unwrap();
        let remaining_time = self.remaining_time(&now);
        let ratio = self.remaining_ratio(&now);
        let label = Line::from(vec![
            Span::raw("Remaining time").add_modifier(Modifier::BOLD),
            Span::raw(" ⏰ "),
            Span::styled(
                format!("{}", remaining_time),
                Style::default().fg(Color::Red),
            ),
        ]);
        let progress = LineGauge::default()
            .filled_style(Style::default().fg(Color::Red))
            .line_set(symbols::line::THICK)
            .label(label)
            .ratio(ratio);
        progress.render(progress_area, buf);

        // in progress downloads
        let items: Vec<ListItem> = self
            .notifications
            .emitted
            .iter()
            .map(|n| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}", n.time), Style::default().fg(Color::Gray)),
                    Span::raw(" ➡ "),
                    Span::styled(
                        format!("{}", n.message),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]))
            })
            .collect();
        let list = List::new(items);
        list.render(main, buf);
    }
}

fn main() -> AppResult<()> {
    init::error_reporting();
    init::tracing();

    let cli = Cli::parse();
    let plan = Plan {
        rendezvous_time: Timestamp::parse_today_time(&cli.rendezvous).change_context(AppError)?,
        trip_duration: TimeSpan::parse(&cli.trip).change_context(AppError)?,
    };

    let coach = DefaultItCoach;
    let mut speaker = TTSSpeaker::new().change_context(AppError)?;

    let mut app = AppState::new(&plan, coach, 10)?;

    // viewport height in lines =
    // 1 (departure time) +
    // 1 (remaining w/ line gauge) +
    // (max number of messages)
    let mut terminal = ratatui::init_with_options(TerminalOptions {
        viewport: Viewport::Inline(2 + app.notifications.max_emitted as u16),
    });

    let result = app.run(&mut terminal, &mut speaker);

    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_just_created_is_a_clean_slate() {
        let plan = Plan {
            rendezvous_time: Timestamp::new(2025, 10, 24, 18, 00, 00).unwrap(),
            trip_duration: TimeSpan::of_minutes(15),
        };
        let state = AppState::new(&plan, DefaultItCoach, 5).unwrap();

        assert!(!state.exit);
        assert!(state.notifications.emitted.is_empty());
    }

    #[test]
    fn notifications_emitted_is_a_ring_with_fixed_capacity() {
        let mut notifications = Notifications::new(vec![], 5);

        for i in 1..=10 {
            notifications.emit(Notification {
                time: Timestamp::now().unwrap(),
                message: format!("{i}"),
            })
        }

        let actual: Vec<_> = notifications
            .emitted
            .into_iter()
            .map(|m| m.message)
            .collect();
        assert_eq!(vec!["10", "9", "8", "7", "6"], actual);
    }
}
