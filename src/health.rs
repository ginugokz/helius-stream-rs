use std::time::{Duration, Instant};

/// Tracks slot continuity, gaps, and freshness of incoming updates.
#[derive(Debug)]
pub struct StreamHealth {
    last_slot: u64,
    last_update: Instant,
    max_observed_gap: u64,
    consecutive_clean: u64,
    total_updates: u64,
    total_gaps: u64,
}

impl StreamHealth {
    pub fn new() -> Self {
        Self {
            last_slot: 0,
            last_update: Instant::now(),
            max_observed_gap: 0,
            consecutive_clean: 0,
            total_updates: 0,
            total_gaps: 0,
        }
    }

    /// Returns true if this update revealed a slot gap.
    pub fn record_update(&mut self, slot: u64) -> bool {
        let gap = if self.last_slot > 0 && slot > self.last_slot + 1 {
            let g = slot - self.last_slot - 1;
            self.max_observed_gap = self.max_observed_gap.max(g);
            self.total_gaps += 1;
            self.consecutive_clean = 0;
            true
        } else {
            self.consecutive_clean += 1;
            false
        };
        self.last_slot = self.last_slot.max(slot);
        self.last_update = Instant::now();
        self.total_updates += 1;
        gap
    }

    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.last_update.elapsed() > max_age
    }

    pub fn last_slot(&self) -> u64 { self.last_slot }
    pub fn total_gaps(&self) -> u64 { self.total_gaps }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn max_observed_gap(&self) -> u64 { self.max_observed_gap }
    pub fn consecutive_clean(&self) -> u64 { self.consecutive_clean }

    pub fn gap_rate(&self) -> f64 {
        if self.total_updates == 0 { return 0.0; }
        self.total_gaps as f64 / self.total_updates as f64
    }

    pub fn last_update(&self) -> Instant { self.last_update }
}

impl Default for StreamHealth {
    fn default() -> Self { Self::new() }
}

/// Exponential backoff with cap, for reconnect attempts.
#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    attempt: u32,
    base_ms: u64,
    max_ms: u64,
}

impl ReconnectPolicy {
    pub fn new() -> Self {
        Self { attempt: 0, base_ms: 100, max_ms: 10_000 }
    }

    pub fn with_bounds(base_ms: u64, max_ms: u64) -> Self {
        Self { attempt: 0, base_ms, max_ms }
    }

    pub fn next_delay_ms(&mut self) -> u64 {
        let cap = self.max_ms.min(self.base_ms * (1u64 << self.attempt.min(10)));
        self.attempt += 1;
        cap / 2
    }

    pub fn reset(&mut self) { self.attempt = 0; }
    pub fn attempt(&self) -> u32 { self.attempt }
}

impl Default for ReconnectPolicy {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_stream_no_gap() {
        let mut h = StreamHealth::new();
        for s in 100..110u64 { assert!(!h.record_update(s)); }
        assert_eq!(h.total_gaps(), 0);
    }

    #[test]
    fn gap_detected_on_slot_skip() {
        let mut h = StreamHealth::new();
        h.record_update(100);
        assert!(h.record_update(105));
        assert_eq!(h.max_observed_gap, 4);
    }

    #[test]
    fn stale_with_zero_threshold() {
        let h = StreamHealth::new();
        assert!(h.is_stale(Duration::from_millis(0)));
    }

    #[test]
    fn backoff_grows_and_caps() {
        let mut r = ReconnectPolicy::new();
        let d0 = r.next_delay_ms();
        let d1 = r.next_delay_ms();
        assert!(d1 >= d0);
        for _ in 0..30 { r.next_delay_ms(); }
        assert!(r.next_delay_ms() <= r.max_ms);
    }

    #[test]
    fn backoff_resets() {
        let mut r = ReconnectPolicy::new();
        for _ in 0..6 { r.next_delay_ms(); }
        let high = r.next_delay_ms();
        r.reset();
        assert!(r.next_delay_ms() < high);
    }

    #[test]
    fn gap_rate_bounded() {
        let mut h = StreamHealth::new();
        for i in 0..10u64 { h.record_update(i * 2); }
        assert!(h.gap_rate() > 0.0 && h.gap_rate() < 1.0);
    }
}
