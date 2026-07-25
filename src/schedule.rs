#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalKind {
    Run,
    Walk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    pub kind: IntervalKind,
    pub duration_secs: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Session {
    pub week: u32,
    pub session: u32,
    pub intervals: &'static [Interval],
}

impl Session {
    /// Calculates the absolute total length of a training session in seconds
    pub fn total_secs(&self) -> u32 {
        self.intervals.iter().map(|i| i.duration_secs).sum()
    }

    /// Calculates what ratio of the workout is running vs walking
    pub fn run_percentage(&self) -> u32 {
        let total = self.total_secs();
        if total == 0 {
            return 0;
        }
        let run_total: u32 = self
            .intervals
            .iter()
            .filter(|i| i.kind == IntervalKind::Run)
            .map(|i| i.duration_secs)
            .sum();

        ((run_total as f32 / total as f32) * 100.0) as u32
    }
}
