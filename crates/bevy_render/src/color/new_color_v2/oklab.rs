use super::{rgb::RGB, Mix};

pub struct Oklab<T> {
    lightness: T,
    a_axis: T,
    b_axis: T,
}

// Const Context
impl<T> Oklab<T>
where
    T: Copy,
{
    pub const fn new(lightness: T, a_axis: T, b_axis: T) -> Self {
        Self {
            lightness,
            a_axis,
            b_axis
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
    
    pub const fn ax(&self) -> T {
        self.a_axis
    }

    pub const fn with_ax(self, a_axis: T) -> Self {
        Self {
            a_axis,
            ..self
        }
    }
    
    pub const fn bx(&self) -> T {
        self.b_axis
    }

    pub const fn with_bx(self, b_axis: T) -> Self {
        Self {
            b_axis,
            ..self
        }
    }
}

impl<T> Oklab<T>
where
    T: Copy,
{
    pub fn set_l(&mut self, l: T) -> &mut Self {
        self.lightness = l;
        self
    }
    
    pub fn set_ax(&mut self, ax: T) -> &mut Self {
        self.a_axis = ax;
        self
    }
    
    pub fn set_bx(&mut self, bx: T) -> &mut Self {
        self.b_axis = bx;
        self
    }
}

impl From<Oklab<f32>> for RGB<f32> {
    fn from(value: Oklab<f32>) -> Self {
        let lightness = value.l();
        let a_axis = value.ax();
        let b_axis = value.bx();

        let l_ = lightness + 0.3963377774 * a_axis + 0.2158037573 * b_axis;
        let m_ = lightness - 0.1055613458 * a_axis - 0.0638541728 * b_axis;
        let s_ = lightness - 0.0894841775 * a_axis - 1.2914855480 * b_axis;

        let l = l_*l_*l_;
        let m = m_*m_*m_;
        let s = s_*s_*s_;

        let red = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
        let green = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
        let blue = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

        let red = red.clamp(0., 1.);
        let green = green.clamp(0., 1.);
        let blue = blue.clamp(0., 1.);

        Self::new(red, green, blue)
    }
}

impl From<RGB<f32>> for Oklab<f32> {
    fn from(value: RGB<f32>) -> Self {
        let red = value.r();
        let green = value.g();
        let blue = value.b();

        debug_assert!(0. <= red && red <= 1.);
        debug_assert!(0. <= green && green <= 1.);
        debug_assert!(0. <= blue && blue <= 1.);
        
        let l = 0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue;
        let m = 0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue;
        let s = 0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        let lightness = 0.2104542553*l_ + 0.7936177850*m_ - 0.0040720468*s_;
        let a_axis = 1.9779984951*l_ - 2.4285922050*m_ + 0.4505937099*s_;
        let b_axis = 0.0259040371*l_ + 0.7827717662*m_ - 0.8086757660*s_;

        Self::new(lightness, a_axis, b_axis)
    }
}

impl Mix for Oklab<f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let lightness = self.lightness.mix(target.lightness, t);
        let a_axis = self.a_axis.mix(target.a_axis, t);
        let b_axis = self.b_axis.mix(target.b_axis, t);

        Self::new(lightness, a_axis, b_axis)
    }
}
