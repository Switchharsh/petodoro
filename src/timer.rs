//! Pomodoro and stopwatch state machines. Pure logic, no I/O.

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn is_break(self) -> bool {
        self != Phase::Focus
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PomoSettings {
    pub focus_min: u32,
    pub short_min: u32,
    pub long_min: u32,
    /// A long break replaces every N-th short break.
    pub long_every: u32,
    /// Start the next phase automatically when one ends.
    pub auto_continue: bool,
}

impl Default for PomoSettings {
    fn default() -> Self {
        PomoSettings { focus_min: 25, short_min: 5, long_min: 15, long_every: 4, auto_continue: false }
    }
}

impl PomoSettings {
    pub fn minutes(&self, p: Phase) -> u32 {
        match p {
            Phase::Focus => self.focus_min,
            Phase::ShortBreak => self.short_min,
            Phase::LongBreak => self.long_min,
        }
    }

    pub fn minutes_mut(&mut self, p: Phase) -> &mut u32 {
        match p {
            Phase::Focus => &mut self.focus_min,
            Phase::ShortBreak => &mut self.short_min,
            Phase::LongBreak => &mut self.long_min,
        }
    }

    pub fn duration(&self, p: Phase) -> Duration {
        Duration::from_secs(60 * self.minutes(p).max(1) as u64)
    }
}

#[derive(Debug)]
pub struct Pomodoro {
    pub phase: Phase,
    /// Focus sessions finished (or skipped) so far.
    pub completed: u32,
    remaining: Duration,
    ends_at: Option<Instant>,
}

impl Pomodoro {
    pub fn new(s: &PomoSettings) -> Self {
        Pomodoro { phase: Phase::Focus, completed: 0, remaining: s.duration(Phase::Focus), ends_at: None }
    }

    pub fn is_running(&self) -> bool {
        self.ends_at.is_some()
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        match self.ends_at {
            Some(end) => end.saturating_duration_since(now),
            None => self.remaining,
        }
    }

    /// Not running and never started in this phase.
    pub fn is_fresh(&self, s: &PomoSettings) -> bool {
        !self.is_running() && self.remaining == s.duration(self.phase)
    }

    pub fn start(&mut self, now: Instant) {
        if self.ends_at.is_none() {
            self.ends_at = Some(now + self.remaining);
        }
    }

    pub fn pause(&mut self, now: Instant) {
        if let Some(end) = self.ends_at.take() {
            self.remaining = end.saturating_duration_since(now);
        }
    }

    pub fn toggle(&mut self, now: Instant) {
        if self.is_running() {
            self.pause(now)
        } else {
            self.start(now)
        }
    }

    /// Restart the current phase from its full length, stopped.
    pub fn reset(&mut self, s: &PomoSettings) {
        self.ends_at = None;
        self.remaining = s.duration(self.phase);
    }

    /// Jump to the next phase (stopped).
    pub fn skip(&mut self, s: &PomoSettings) {
        self.advance(s);
    }

    fn advance(&mut self, s: &PomoSettings) {
        self.phase = match self.phase {
            Phase::Focus => {
                self.completed += 1;
                if self.completed % s.long_every.max(1) == 0 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Focus,
        };
        self.reset(s);
    }

    /// Advance time. Returns the phase that just finished, if any.
    pub fn tick(&mut self, s: &PomoSettings, now: Instant) -> Option<Phase> {
        let end = self.ends_at?;
        if now < end {
            return None;
        }
        let finished = self.phase;
        self.advance(s);
        if s.auto_continue {
            self.start(now);
        }
        Some(finished)
    }

    /// 0.0 at the start of the phase, 1.0 at the end.
    pub fn progress(&self, s: &PomoSettings, now: Instant) -> f32 {
        let total = s.duration(self.phase).as_secs_f32();
        (1.0 - self.remaining(now).as_secs_f32() / total).clamp(0.0, 1.0)
    }

    /// 1-based index of the current focus session inside the long-break cycle.
    pub fn session_in_cycle(&self, s: &PomoSettings) -> u32 {
        let every = s.long_every.max(1);
        match self.phase {
            Phase::Focus => self.completed % every + 1,
            // During a break we show the session that was just finished.
            _ => (self.completed + every - 1) % every + 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct Stopwatch {
    acc: Duration,
    started: Option<Instant>,
}

impl Stopwatch {
    pub fn is_running(&self) -> bool {
        self.started.is_some()
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        self.acc + self.started.map_or(Duration::ZERO, |s| now.saturating_duration_since(s))
    }

    pub fn toggle(&mut self, now: Instant) {
        match self.started.take() {
            Some(s) => self.acc += now.saturating_duration_since(s),
            None => self.started = Some(now),
        }
    }

    pub fn reset(&mut self) {
        self.acc = Duration::ZERO;
        self.started = None;
    }
}

/// "MM:SS", or "H:MM:SS" past an hour. Countdowns round up so that the
/// display reads 00:00 exactly when the phase ends.
pub fn format_clock(d: Duration, round_up: bool) -> String {
    let ms = d.as_millis() as u64;
    let secs = if round_up { ms.div_ceil(1000) } else { ms / 1000 };
    let (h, m, s) = (secs / 3600, (secs / 60) % 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn pomodoro_counts_down_and_pauses() {
        let s = PomoSettings::default();
        let t0 = Instant::now();
        let mut p = Pomodoro::new(&s);
        assert!(p.is_fresh(&s));
        p.toggle(t0);
        assert_eq!(p.remaining(t0 + secs(60)), secs(24 * 60));
        p.toggle(t0 + secs(60));
        assert!(!p.is_running());
        assert!(!p.is_fresh(&s));
        // Time passing while paused does not count.
        assert_eq!(p.remaining(t0 + secs(600)), secs(24 * 60));
        p.toggle(t0 + secs(600));
        assert_eq!(p.remaining(t0 + secs(660)), secs(23 * 60));
    }

    #[test]
    fn pomodoro_cycle_with_long_break() {
        let s = PomoSettings { focus_min: 1, short_min: 1, long_min: 2, long_every: 2, auto_continue: true };
        let mut now = Instant::now();
        let mut p = Pomodoro::new(&s);
        p.start(now);
        let mut seen = vec![];
        for _ in 0..4 {
            now += p.remaining(now);
            seen.push(p.tick(&s, now).unwrap());
        }
        assert_eq!(seen, [Phase::Focus, Phase::ShortBreak, Phase::Focus, Phase::LongBreak]);
        assert_eq!(p.phase, Phase::Focus);
        assert!(p.is_running());
        assert_eq!(p.completed, 2);
    }

    #[test]
    fn pomodoro_stops_between_phases_without_auto_continue() {
        let s = PomoSettings::default();
        let t0 = Instant::now();
        let mut p = Pomodoro::new(&s);
        p.start(t0);
        assert_eq!(p.tick(&s, t0 + secs(10)), None);
        assert_eq!(p.tick(&s, t0 + secs(25 * 60)), Some(Phase::Focus));
        assert_eq!(p.phase, Phase::ShortBreak);
        assert!(!p.is_running());
        assert_eq!(p.remaining(t0 + secs(9999)), secs(5 * 60));
        assert_eq!(p.session_in_cycle(&s), 1);
    }

    #[test]
    fn stopwatch_accumulates() {
        let t0 = Instant::now();
        let mut w = Stopwatch::default();
        w.toggle(t0);
        w.toggle(t0 + secs(5));
        w.toggle(t0 + secs(100));
        assert_eq!(w.elapsed(t0 + secs(103)), secs(8));
        w.reset();
        assert_eq!(w.elapsed(t0 + secs(200)), Duration::ZERO);
    }

    #[test]
    fn clock_format() {
        assert_eq!(format_clock(Duration::from_millis(1500), true), "00:02");
        assert_eq!(format_clock(Duration::from_millis(1500), false), "00:01");
        assert_eq!(format_clock(secs(25 * 60), true), "25:00");
        assert_eq!(format_clock(secs(3723), false), "1:02:03");
    }
}
