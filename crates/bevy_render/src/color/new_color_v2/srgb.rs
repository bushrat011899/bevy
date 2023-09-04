use super::{rgb::RGB, Mix};

pub struct SRGB<T> {
    red: T,
    green: T,
    blue: T,
}

// Const Context
impl<T> SRGB<T>
where
    T: Copy,
{
    pub const fn new(red: T, green: T, blue: T) -> Self {
        Self {
            red,
            green,
            blue
        }
    }

    pub const fn r(&self) -> T {
        self.red
    }

    pub const fn with_r(self, red: T) -> Self {
        Self {
            red,
            ..self
        }
    }
    
    pub const fn g(&self) -> T {
        self.green
    }

    pub const fn with_g(self, green: T) -> Self {
        Self {
            green,
            ..self
        }
    }
    
    pub const fn b(&self) -> T {
        self.blue
    }

    pub const fn with_b(self, blue: T) -> Self {
        Self {
            blue,
            ..self
        }
    }
}

impl<T> SRGB<T>
where
    T: Copy,
{
    pub fn set_r(&mut self, r: T) -> &mut Self {
        self.red = r;
        self
    }
    
    pub fn set_g(&mut self, g: T) -> &mut Self {
        self.green = g;
        self
    }
    
    pub fn set_b(&mut self, b: T) -> &mut Self {
        self.blue = b;
        self
    }
}

impl From<SRGB<f32>> for RGB<f32> {
    fn from(value: SRGB<f32>) -> Self {
        let red = value.r();
        let green = value.g();
        let blue = value.b();

        let red = reverse_gamma_correction(red);
        let green = reverse_gamma_correction(green);
        let blue = reverse_gamma_correction(blue);

        Self::new(red, green, blue)
    }
}

impl From<RGB<f32>> for SRGB<f32> {
    fn from(value: RGB<f32>) -> Self {
        let red = value.r();
        let green = value.g();
        let blue = value.b();

        let red = gamma_correction(red);
        let green = gamma_correction(green);
        let blue = gamma_correction(blue);

        Self::new(red, green, blue)
    }
}

impl Mix for SRGB<f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let red = self.red.mix(target.red, t);
        let green = self.green.mix(target.green, t);
        let blue = self.blue.mix(target.blue, t);

        Self::new(red, green, blue)
    }
}

fn gamma_correction(value: f32) -> f32 {
    debug_assert!(0. <= value && value <= 1.);

    const A: f32 = 1.055;
    const B: f32 = -0.055;
    const C: f32 = 12.92;
    const D: f32 = 0.0031308;
    const Y: f32 = 1. / 2.4;

    let result = if value < D {
        value * C
    } else {
        A * (value.powf(Y)) + B
    };

    debug_assert!(0. <= result && result <= 1.);

    result
}

fn reverse_gamma_correction(value: f32) -> f32 {
    debug_assert!(0. <= value && value <= 1.);

    const A: f32 = 1.055;
    const B: f32 = -0.055;
    const C: f32 = 12.92;
    const D: f32 = 0.0031308;
    const Y_: f32 = 2.4;

    let result = value / C;

    let result = if result < D {
        result
    } else {
        ((value - B) / A).powf(Y_)
    };

    debug_assert!(0. <= result && result <= 1.);

    result
}