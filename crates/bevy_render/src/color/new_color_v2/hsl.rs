use super::{rgb::RGB, Mix};

pub struct HSL<T> {
    hue: T,
    saturation: T,
    lightness: T,
}

// Const Context
impl<T> HSL<T>
where
    T: Copy,
{
    pub const fn new(hue: T, saturation: T, lightness: T) -> Self {
        Self {
            hue,
            saturation,
            lightness
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

    pub const fn l(&self) -> T {
        self.lightness
    }

    pub const fn with_l(self, lightness: T) -> Self {
        Self {
            lightness,
            ..self
        }
    }
}

impl<T> HSL<T>
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
    
    pub fn set_l(&mut self, l: T) -> &mut Self {
        self.lightness = l;
        self
    }
}

impl From<HSL<f32>> for RGB<f32> {
    fn from(value: HSL<f32>) -> Self {
        fn hue_to_channel(v1: f32, v2: f32, v_h: f32) -> f32 {
            let v_h = if v_h < 0. {
                v_h + 1.
            } else if v_h > 1. {
                v_h - 1.
            } else {
                v_h
            };

            if 6. * v_h < 1. {
                v1 + (v2 - v1) * 6. * v_h
            } else if 2. * v_h < 1. {
                v2
            } else if 3. * v_h < 2. {
                v1 + (v2 - v1) * ((2. / 3.) - v_h) * 6.
            } else {
                v1
            }
        }

        let hue = value.h();
        let saturation = value.s();
        let lightness = value.l();

        if saturation == 0. {
            Self::new(lightness, lightness, lightness)
        } else {
            let v2 = if lightness < 0.5 {
                lightness * ( 1. + saturation )
            } else {
                (lightness + saturation) - (saturation * lightness)
            };

            let v1 = 2. * lightness - v2;

            let red = hue_to_channel(v1, v2, hue + (1. / 3.));
            let green = hue_to_channel(v1, v2, hue);
            let blue = hue_to_channel(v1, v2, hue - (1. / 3.));

            debug_assert!(0. <= red && red <= 1.);
            debug_assert!(0. <= green && green <= 1.);
            debug_assert!(0. <= blue && blue <= 1.);

            Self::new(red, green, blue)
        }
    }
}

impl From<RGB<f32>> for HSL<f32> {
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

        let lightness = (channel_max + channel_min) / 2.;

        let saturation = if lightness < 0.5 {
            channel_delta / (channel_max + channel_min)
        } else {
            channel_delta / (2. - channel_max - channel_min)
        };

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

        Self::new(hue, saturation, lightness)
    }
}

impl Mix for HSL<f32> {
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
        let lightness = self.lightness.mix(target.lightness, t);

        Self::new(hue, saturation, lightness)
    }
}
