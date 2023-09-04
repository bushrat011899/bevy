use super::{oklab::Oklab, Mix};

pub struct Oklch<T> {
    lightness: T,
    chroma: T,
    hue: T,
}

// Const Context
impl<T> Oklch<T>
where
    T: Copy,
{
    pub const fn new(lightness: T, chroma: T, hue: T) -> Self {
        Self {
            lightness,
            chroma,
            hue
        }
    }

    pub const fn l(&self) -> T {
        self.lightness
    }

    pub const fn with_l(self, lightness: T) -> Self {
        Self {
            lightness,
            ..self
        }
    }
    
    pub const fn c(&self) -> T {
        self.chroma
    }

    pub const fn with_c(self, chroma: T) -> Self {
        Self {
            chroma,
            ..self
        }
    }
    
    pub const fn h(&self) -> T {
        self.hue
    }

    pub const fn with_h(self, hue: T) -> Self {
        Self {
            hue,
            ..self
        }
    }
}

impl<T> Oklch<T>
where
    T: Copy,
{
    pub fn set_l(&mut self, l: T) -> &mut Self {
        self.lightness = l;
        self
    }
    
    pub fn set_c(&mut self, c: T) -> &mut Self {
        self.chroma = c;
        self
    }
    
    pub fn set_h(&mut self, h: T) -> &mut Self {
        self.hue = h;
        self
    }
}

impl From<Oklch<f32>> for Oklab<f32> {
    fn from(value: Oklch<f32>) -> Self {
        let lightness = value.l();
        let chroma = value.c();
        let hue = value.h();

        let a_axis = hue.cos() * chroma;
        let b_axis = hue.sin() * chroma;

        Self::new(lightness, a_axis, b_axis)
    }
}

impl From<Oklab<f32>> for Oklch<f32> {
    fn from(value: Oklab<f32>) -> Self {
        let lightness = value.l();
        let a = value.ax();
        let b = value.bx();

        let chroma = a.hypot(b);
        let hue = b.atan2(a);

        Self::new(lightness, chroma, hue)
    }
}

impl Mix for Oklch<f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        use std::f32::consts::PI;
        
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let lightness = self.lightness.mix(target.lightness, t);
        let chroma = self.chroma.mix(target.chroma, t);

        let hue = {
            if (target.hue - self.hue).abs() <= PI {
                self.hue.mix(target.hue, t)
            } else {
                if target.hue > self.hue {
                    self.hue.mix(target.hue - 2. * PI, t)
                } else {
                    self.hue.mix(target.hue + 2. * PI, t)
                }
            }
        };

        let hue = if hue < -PI {
            hue + 2. * PI
        } else if hue > PI {
            hue - 2. * PI
        } else {
            hue
        };

        Self::new(lightness, chroma, hue)
    }
}
