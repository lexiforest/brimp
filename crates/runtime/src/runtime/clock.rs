//! Realm time is independent of the real deadlines used for I/O and cancellation.
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeMode {
    #[default]
    System,
    Logical,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ClockOptions {
    pub mode: TimeMode,
    /// Unix epoch in milliseconds. Logical time defaults to zero.
    pub epoch_ms: Option<u64>,
}

pub(crate) struct Clock {
    mode: TimeMode,
    start: Instant,
    elapsed: Duration,
    origin_ms: f64,
}
impl Clock {
    pub fn new(options: &ClockOptions) -> Self {
        Self {
            mode: options.mode,
            start: Instant::now(),
            elapsed: Duration::ZERO,
            origin_ms: options.epoch_ms.map(|n| n as f64).unwrap_or_else(|| {
                if options.mode == TimeMode::Logical {
                    0.0
                } else {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs_f64()
                        * 1000.0
                }
            }),
        }
    }
    pub fn logical(&self) -> bool {
        self.mode == TimeMode::Logical
    }
    pub fn now(&self) -> Duration {
        if self.logical() {
            self.elapsed
        } else {
            self.start.elapsed()
        }
    }
    pub fn origin_ms(&self) -> f64 {
        self.origin_ms
    }
    pub fn advance_to(&mut self, time: Duration) {
        self.elapsed = self.elapsed.max(time);
    }
}
