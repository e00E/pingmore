use std::time::{Duration, Instant};

pub fn deadline_to_timeout(deadline: Option<Instant>) -> Option<Duration> {
    deadline.map(|d| d.saturating_duration_since(Instant::now()))
}
