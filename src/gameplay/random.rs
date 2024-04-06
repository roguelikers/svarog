use std::ops::{Range, RangeInclusive};

use bevy::app::Plugin;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand, resource::GlobalEntropy};
use rand_core::RngCore;

pub struct SvarogRandomPlugin;

pub type Random = GlobalEntropy<WyRand>;

pub trait Coin {
    fn coin(&mut self) -> bool;
}

pub trait RNG<T> {
    fn between(&mut self, range: RangeInclusive<T>) -> T;
    fn strict_between(&mut self, range: Range<T>) -> T;
}

pub trait Choice<T> {
    fn from(&mut self, source: &[T]) -> T;
}

impl Coin for Random {
    fn coin(&mut self) -> bool {
        (self.next_u32() % 100) >= 50
    }
}

impl RNG<i32> for Random {
    fn between(&mut self, range: RangeInclusive<i32>) -> i32 {
        let a = *range.start();
        let b = *range.end();
        let d = b - a;
        a + (self.next_u32() as i32 % d)
    }

    fn strict_between(&mut self, range: Range<i32>) -> i32 {
        let a = range.start;
        let b = range.end;
        let d = b - a;
        a + (self.next_u32() as i32 % d)
    }
}

impl RNG<f32> for Random {
    fn between(&mut self, range: RangeInclusive<f32>) -> f32 {
        let a = *range.start();
        let b = *range.end();
        let d = b - a;
        a + (self.next_u32() as f32 % d)
    }

    fn strict_between(&mut self, range: Range<f32>) -> f32 {
        let a = range.start;
        let b = range.end;
        let d = b - a;
        a + (self.next_u32() as f32 % d)
    }
}

impl<T: Clone> Choice<T> for Random {
    fn from(&mut self, source: &[T]) -> T {
        let n = self.next_u32() as usize;
        source[n % source.len()].clone()
    }
}

impl Plugin for SvarogRandomPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EntropyPlugin::<WyRand>::default());
    }
}