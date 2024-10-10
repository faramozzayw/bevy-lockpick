use bevy::prelude::*;
use rand::Rng;

use crate::{FAST_RISE_IN_SECS, RISE_NUM, SLOW_RISE_IN_SECS};

#[derive(Component)]
pub struct AutoAttemptButton;

#[derive(Component)]
pub struct Lock;

#[derive(Component)]
pub struct LockpickLabel;

#[derive(Component)]
pub struct Lockpick {
    pub current_position: usize,
}

#[derive(Component, Deref)]
pub struct Spring(pub usize);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Pin {
    pub rise_time: f32,
    pub index: usize,
    pub next_rise: usize,
    pub pattern: Vec<Rise>,
}

#[derive(Debug, Clone, Copy, Reflect)]
pub enum Rise {
    Slow,
    Fast,
}

impl Pin {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            next_rise: 0,
            rise_time: 0.0,
            pattern: Self::generate_rise_pattern(RISE_NUM),
        }
    }

    fn generate_rise_pattern(n: usize) -> Vec<Rise> {
        (0..n)
            .map(|_| {
                if rand::thread_rng().gen_bool(0.5) {
                    Rise::Fast
                } else {
                    Rise::Slow
                }
            })
            .collect()
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn inc_next_rise(&mut self) {
        self.next_rise += 1;

        if self.next_rise == self.pattern.len() {
            self.next_rise = 0;
        }
    }

    pub fn is_time_limit_reached(&self) -> bool {
        let rise_time = self.get_current_rise_time();
        rise_time <= self.rise_time
    }

    pub fn get_current_rise_time(&self) -> f32 {
        match self.pattern[self.next_rise] {
            Rise::Slow => SLOW_RISE_IN_SECS,
            Rise::Fast => FAST_RISE_IN_SECS,
        }
    }

    fn standardize_time(current_seconds: f32, total_seconds: f32) -> f32 {
        if total_seconds <= 0.0 {
            return 0.0;
        }
        (current_seconds / total_seconds).clamp(0.0, 1.0)
    }

    fn progress(t: f32) -> f32 {
        if 1.0 <= t {
            1.0
        } else {
            1.0 - 2f32.powf(-16.0 * t)
        }
    }

    #[inline]
    pub fn get_progress(&self) -> f32 {
        Self::progress(Self::standardize_time(
            self.rise_time,
            self.get_current_rise_time(),
        ))
    }
}

#[derive(Component, Default)]
pub struct LockedPin;

#[derive(Component, Default)]
pub struct UnlockedPin;

#[derive(Component, Default)]
pub struct UnlockedByDefaultPin;

#[derive(Component, Default)]
pub struct TriggeredPin;

#[derive(Component, Default)]
pub struct DroppingPin;
