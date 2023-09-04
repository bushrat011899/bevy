use super::Mix;

pub struct RGB<T> {
    red: T,
    green: T,
    blue: T,
}

// Const Context
impl<T> RGB<T>
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

impl<T> RGB<T>
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

impl Mix for RGB<f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let red = self.red.mix(target.red, t);
        let green = self.green.mix(target.green, t);
        let blue = self.blue.mix(target.blue, t);

        Self::new(red, green, blue)
    }
}

impl Mix for RGB<u8> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let red = self.red.mix(target.red, t);
        let green = self.green.mix(target.green, t);
        let blue = self.blue.mix(target.blue, t);

        Self::new(red, green, blue)
    }
}

impl From<RGB<u8>> for RGB<f32> {
    fn from(value: RGB<u8>) -> Self {
        const MAX: f32 = u8::MAX as f32;

        let red = value.r();
        let green = value.g();
        let blue = value.b();

        let red = (red as f32) / MAX;
        let green = (green as f32) / MAX;
        let blue = (blue as f32) / MAX;

        Self::new(red, green, blue)
    }
}

impl From<RGB<f32>> for RGB<u8> {
    fn from(value: RGB<f32>) -> Self {
        const MAX: f32 = u8::MAX as f32;

        let red = value.r();
        let green = value.g();
        let blue = value.b();

        let red = (red * MAX).clamp(0., MAX).floor() as u8;
        let green = (green * MAX).clamp(0., MAX).floor() as u8;
        let blue = (blue * MAX).clamp(0., MAX).floor() as u8;

        Self::new(red, green, blue)
    }
}