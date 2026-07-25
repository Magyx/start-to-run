use crate::schedule::{IntervalKind, Session};

#[derive(Debug, Clone, Copy)]
pub struct TimerState {
    pub current_index: usize,
    pub current_elapsed: u32,
    pub total_elapsed: u32,
    pub running: bool,
    pub done: bool,
}

impl TimerState {
    /// Creates a fresh, idle timer state
    pub fn new() -> Self {
        Self {
            current_index: 0,
            current_elapsed: 0,
            total_elapsed: 0,
            running: false,
            done: false,
        }
    }

    /// Toggles between active countdown and paused states
    pub fn toggle(&mut self) {
        if !self.done {
            self.running = !self.running;
        }
    }

    /// Resets the state machine back to zero
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Advances the timer by 1 second.
    /// Returns `true` if an interval transitioned (useful for playing a audio alert/beep).
    pub fn tick(&mut self, session: &Session) -> bool {
        if self.done || !self.running {
            return false;
        }

        self.current_elapsed += 1;
        self.total_elapsed += 1;

        // Fetch the active interval split configuration
        if let Some(current_interval) = session.intervals.get(self.current_index) {
            // Check if the current block completed its countdown allocation
            if self.current_elapsed >= current_interval.duration_secs {
                self.current_index += 1;
                self.current_elapsed = 0;

                // Check if that was the final block of the training routine
                if self.current_index >= session.intervals.len() {
                    self.done = true;
                    self.running = false;
                }
                return true; // Return true to indicate an interval boundary has flipped
            }
        } else {
            // Safeguard: index out of bounds completes the workout
            self.done = true;
            self.running = false;
            return true;
        }

        false
    }

    /// Identifies the active workout mode (Running vs Walking)
    pub fn current_kind(&self, session: &Session) -> IntervalKind {
        session
            .intervals
            .get(self.current_index)
            .map(|i| i.kind)
            .unwrap_or(IntervalKind::Walk)
    }

    /// Calculates seconds remaining in the active block
    pub fn interval_remaining(&self, session: &Session) -> u32 {
        if self.done {
            return 0;
        }
        session
            .intervals
            .get(self.current_index)
            .map(|i| i.duration_secs.saturating_sub(self.current_elapsed))
            .unwrap_or(0)
    }

    /// Computes overall workout completion progress from 0.0 to 1.0
    pub fn progress(&self, session: &Session) -> f32 {
        let total_duration: u32 = session.intervals.iter().map(|i| i.duration_secs).sum();
        if total_duration == 0 {
            return 1.0;
        }
        (self.total_elapsed as f32 / total_duration as f32).min(1.0)
    }

    /// Calculates seconds remaining until the absolute end of the training session
    pub fn total_remaining(&self, session: &Session) -> u32 {
        let total_duration: u32 = session.intervals.iter().map(|i| i.duration_secs).sum();
        total_duration.saturating_sub(self.total_elapsed)
    }

    /// Returns a preview tuple of the up-and-coming interval split
    pub fn next_interval(&self, session: &Session) -> Option<(IntervalKind, u32)> {
        session
            .intervals
            .get(self.current_index + 1)
            .map(|i| (i.kind, i.duration_secs))
    }
}

/// Utility helper to translate raw seconds into digital clock text (MM:SS)
pub fn format_time(seconds: u32) -> String {
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, remaining_seconds)
}
