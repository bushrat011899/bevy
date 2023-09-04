use super::{rgb::RGB, Mix};

pub struct XYZ<T> {
    x: T,
    y: T,
    z: T,
}

// Const Context
impl<T> XYZ<T>
where
    T: Copy,
{
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z
        }
    }

    pub const fn x(&self) -> T {
        self.x
    }

    pub const fn with_x(self, x: T) -> Self {
        Self {
            x,
            ..self
        }
    }
    
    pub const fn y(&self) -> T {
        self.y
    }

    pub const fn with_y(self, y: T) -> Self {
        Self {
            y,
            ..self
        }
    }
    
    pub const fn z(&self) -> T {
        self.z
    }

    pub const fn with_z(self, z: T) -> Self {
        Self {
            z,
            ..self
        }
    }
}

impl<T> XYZ<T>
where
    T: Copy,
{
    pub fn set_x(&mut self, x: T) -> &mut Self {
        self.x = x;
        self
    }
    
    pub fn set_y(&mut self, y: T) -> &mut Self {
        self.y = y;
        self
    }
    
    pub fn set_z(&mut self, z: T) -> &mut Self {
        self.z = z;
        self
    }
}

impl From<XYZ<f32>> for RGB<f32> {
    fn from(value: XYZ<f32>) -> Self {
        let x = value.x();
        let y = value.y();
        let z = value.z();

        let x = x / 100.;
        let y = y / 100.;
        let z = z / 100.;

        let red = x *  3.2406 + y * -1.5372 + z * -0.4986;
        let green = x * -0.9689 + y *  1.8758 + z *  0.0415;
        let blue = x *  0.0557 + y * -0.2040 + z *  1.0570;

        let red = red.clamp(0., 1.);
        let green = green.clamp(0., 1.);
        let blue = blue.clamp(0., 1.);

        Self::new(red, green, blue)
    }
}

impl From<RGB<f32>> for XYZ<f32> {
    fn from(value: RGB<f32>) -> Self {
        let red = value.r();
        let green = value.g();
        let blue = value.b();
        
        debug_assert!(0. <= red && red <= 1.);
        debug_assert!(0. <= green && green <= 1.);
        debug_assert!(0. <= blue && blue <= 1.);

        let red = 100. * red;
        let green = 100. * green;
        let blue = 100. * blue;

        let x = red * 0.4124 + green * 0.3576 + blue * 0.1805;
        let y = red * 0.2126 + green * 0.7152 + blue * 0.0722;
        let z = red * 0.0193 + green * 0.1192 + blue * 0.9505;

        Self::new(x, y, z)
    }
}

impl Mix for XYZ<f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let x = self.x.mix(target.x, t);
        let y = self.y.mix(target.y, t);
        let z = self.z.mix(target.z, t);

        Self::new(x, y, z)
    }
}