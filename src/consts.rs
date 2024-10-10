pub const RISE_NUM: usize = 4;
pub const UNLOCKED_BY_DEFAULT_NUM: usize = 2;
pub const DROP_NUM_AFTER_FAIL: usize = 2;

pub const AUTO_ATTEMPT_CHANCE: f32 = 0.05;

pub const MAX_LOCKPICK_BOTTOM_PERCENT: f32 = 170.0;
pub const MIN_LOCKPICK_BOTTOM_PERCENT: f32 = 145.0;
pub const LOCKPICK_SPEED_PER_MS: f32 = 0.1;

pub const SWEET_SPOT_STARTS_AT: f32 = 35.0;

pub const MAX_PIN_BOTTOM_PERCENT: f32 = 40.0;
pub const MIN_PIN_BOTTOM_PERCENT: f32 = 0.0;
pub const TOTAL_PIN_CHANGE: f32 = MAX_PIN_BOTTOM_PERCENT - MIN_PIN_BOTTOM_PERCENT;

pub const MIN_SPRING_HEIGHT_PERCENT: f32 = 0.0;
pub const MAX_SPRING_HEIGHT_PERCENT: f32 = 45.0;
pub const TOTAL_SPRING_CHANGE: f32 = MAX_SPRING_HEIGHT_PERCENT - MIN_SPRING_HEIGHT_PERCENT;

pub const SLOW_RISE_IN_SECS: f32 = 1.0;
pub const FAST_RISE_IN_SECS: f32 = 0.2;
pub const FALL_DURATION_IN_SECS: f32 = 0.1;

pub const FALL_SHIFT_PER_MS: f32 = TOTAL_PIN_CHANGE / (FALL_DURATION_IN_SECS * 1000.0);

pub const LOCKPICK_LOSS_CHANCE: f32 = 0.6;
pub const TUMBLERS: usize = 6;

pub const FIRST_TUMBLER_POSITION: f32 = -300.0;
pub const TUMBLER_STEP: f32 = 55.5;

pub const LOCKPICK_POSITIONS: [f32; TUMBLERS] = {
    let mut arr = [0.0; TUMBLERS];
    let mut i = 0;
    while i < TUMBLERS {
        arr[i] = FIRST_TUMBLER_POSITION + (i as f32) * TUMBLER_STEP;
        i += 1;
    }
    arr
};
