use super::{rgb::RGB, Mix};

pub struct HSV<T> {
    hue: T,
    saturation: T,
    value: T,
}

// Const Context
impl<T> HSV<T>
where
    T: Copy,
{
    pub const fn new(hue: T, saturation: T, value: T) -> Self {
        Self {
            hue,
            saturation,
            value
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
    
    pub const fn s(&self) -> T {
        self.saturation
    }

    pub const fn with_s(self, saturation: T) -> Self {
        Self {
            saturation,
            ..self
        }
    }

    pub const fn v(&self) -> T {
        self.value
    }

    pub const fn with_v(self, value: T) -> Self {
        Self {
            value,
            ..self
        }
    }
}

impl<T> HSV<T>
where
    T: Copy,
{
    pub fn set_h(&mut self, h: T) -> &mut Self {
        self.hue = h;
        self
    }
    
    pub fn set_s(&mut self, s: T) -> &mut Self {
        self.saturation = s;
        self
    }
    
    pub fn set_v(&mut self, v: T) -> &mut Self {
        self.value = v;
        self
    }
}

impl From<HSV<f32>> for RGB<f32> {
    fn from(value: HSV<f32>) -> Self {
        let hue = value.h();
        let saturation = value.s();
        let value = value.v();

        if saturation == 0. {
            Self::new(value, value, value)
        } else {
            let hue = if hue == 1. { 0. } else { hue * 6. };

            let i = hue.floor();
            let v1 = value * (1. - saturation);
            let v2 = value * (1. - saturation * (hue - i));
            let v3 = value * (1. - saturation * (1. - (hue - i)));

            let (red, green, blue) = match i as u8 {
                0 => (value, v3, v1),
                1 => (v2, value, v1),
                2 => (v1, value, v3),
                3 => (v1, v2, value),
                4 => (v3, v1, value),
                _ => (value, v1, v2),
            };

            debug_assert!(0. <= red && red <= 1.);
            debug_assert!(0. <= green && green <= 1.);
            debug_assert!(0. <= blue && blue <= 1.);

            Self::new(red, green, blue)
        }
    }
}

impl From<RGB<f32>> for HSV<f32> {
    fn from(value: RGB<f32>) -> Self {
        let red = value.r();
        let green = value.g();
        let blue = value.b();

        debug_assert!(0. <= red && red <= 1.);
        debug_assert!(0. <= green && green <= 1.);
        debug_assert!(0. <= blue && blue <= 1.);

        let channel_min = red.min(green).min(blue);
        let channel_max = red.max(green).max(blue);
        let channel_delta = channel_max - channel_min;

        let value = channel_max;

        let saturation = channel_delta / value;

        let hue = if channel_delta != 0. {
            let red_delta = (((channel_max - red) / 6.) + (channel_delta / 2.)) / channel_delta;
            let green_delta = (((channel_max - green) / 6.) + (channel_delta / 2.)) / channel_delta;
            let blue_delta = (((channel_max - blue) / 6.) + (channel_delta / 2.)) / channel_delta;

            let hue = if red_delta == channel_max {
                blue_delta - green_delta
            } else if green_delta == channel_max {
                ( 1. / 3. ) + red_delta - blue_delta
            } else if blue_delta == channel_max {
                ( 2. / 3. ) + green_delta - red_delta
            } else {
                unreachable!("At least one of Red, Green, and Blue must be the largest.")
            };

            let hue = if hue < 0. {
                hue + 1.
            } else if hue > 1. {
                hue - 1.
            } else {
                hue
            };

            hue
        } else {
            0.
        };

        Self::new(hue, saturation, value)
    }
}

impl Mix for HSV<f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let hue = {
            if (target.hue - self.hue).abs() <= 0.5 {
                self.hue.mix(target.hue, t)
            } else {
                if target.hue > self.hue {
                    self.hue.mix(target.hue - 1., t)
                } else {
                    self.hue.mix(target.hue + 1., t)
                }
            }
        };

        let hue = if hue < 0. {
            hue + 1.
        } else if hue > 1. {
            hue - 1.
        } else {
            hue
        };

        let saturation = self.saturation.mix(target.saturation, t);
        let value = self.value.mix(target.value, t);

        Self::new(hue, saturation, value)
    }
}