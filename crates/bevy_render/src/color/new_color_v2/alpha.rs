use std::ops::{Deref, DerefMut};

use super::{Mix, rgb::RGB};

pub struct Transparent<C, T> {
    color: C,
    alpha: T,
}

// Const Context
impl<C, T> Transparent<C, T>
where
    T: Copy,
{
    pub const fn new(color: C, alpha: T) -> Self {
        Self {
            color,
            alpha,
        }
    }

    pub const fn a(&self) -> T {
        self.alpha
    }

    pub const fn with_a(mut self, a: T) -> Self {
        self.alpha = a;
        self
    }

    pub fn set_a(&mut self, a: T) -> &mut Self {
        self.alpha = a;
        self
    }

    pub fn color(self) -> C {
        self.color
    }

    pub fn with_color(mut self, color: C) -> Self {
        self.color = color;
        self
    }

    pub fn set_color(&mut self, color: C) -> &mut Self {
        self.color = color;
        self
    }
}

impl<C, T> Deref for Transparent<C, T> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.color
    }
}

impl<C, T> DerefMut for Transparent<C, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.color
    }
}

impl<C> From<C> for Transparent<C, f32> {
    fn from(value: C) -> Self {
        Self::new(value, 1.0)
    }
}

impl<C: Mix> Mix for Transparent<C, f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let color = self.color.mix(target.color, t);
        let alpha = self.alpha.mix(target.alpha, t);

        Self::new(color, alpha)
    }
}

impl<C: Mix> Mix for Transparent<C, u8> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let color = self.color.mix(target.color, t);
        let alpha = self.alpha.mix(target.alpha, t);

        Self::new(color, alpha)
    }
}

impl<C> From<Transparent<C, f32>> for Transparent<C, u8> {
    fn from(value: Transparent<C, f32>) -> Self {
        const MAX: f32 = u8::MAX as f32;

        let alpha = value.a();
        let color = value.color();

        let alpha = (alpha * MAX).clamp(0., MAX).floor() as u8;

        Self::new(color, alpha)
    }
}

impl<C> From<Transparent<C, u8>> for Transparent<C, f32> {
    fn from(value: Transparent<C, u8>) -> Self {
        const MAX: f32 = u8::MAX as f32;

        let alpha = value.a();
        let color = value.color();

        let alpha = (alpha as f32) * MAX;

        Self::new(color, alpha)
    }
}
