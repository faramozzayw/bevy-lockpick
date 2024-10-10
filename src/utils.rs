use std::collections::HashSet;

use bevy::prelude::*;
use rand::seq::SliceRandom;

pub fn random_indexes(n: usize, m: usize, exclude: &[usize]) -> HashSet<usize> {
    let mut rng = rand::thread_rng();
    let mut indices: Vec<usize> = (0..m).filter(|i| !exclude.contains(i)).collect();
    indices.shuffle(&mut rng);
    indices.into_iter().take(n).collect()
}

pub fn val_as_percent(val: &Val) -> f32 {
    match val {
        Val::Percent(v) => *v,
        _ => unimplemented!(),
    }
}

pub fn val_as_px(val: &Val) -> f32 {
    match val {
        Val::Px(v) => *v,
        _ => unimplemented!(),
    }
}
