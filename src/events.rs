use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct TriggerPin(pub usize);

#[derive(Debug, Default, Event)]
pub struct CheckWin;

#[derive(Debug, Default, Event)]
pub struct TryUnlockPin;
