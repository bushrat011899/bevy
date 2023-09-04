use super::{rgb::RGB, Mix, xyz::XYZ};

pub struct LAB<T> {
    lightness: T,
    a_axis: T,
    b_axis: T,
}

// Const Context
impl<T> LAB<T>
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

impl<T> LAB<T>
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

// CIE Constants
// http://brucelindbloom.com/index.html?LContinuity.html (16) (17)
const CIE_EPSILON: f32 = 216.0 / 24389.0;
const CIE_KAPPA: f32 = 24389.0 / 27.0;
// D65 White Reference:
// https://en.wikipedia.org/wiki/Illuminant_D65#Definition
const D65_WHITE_X: f32 = 0.95047;
const D65_WHITE_Y: f32 = 1.0;
const D65_WHITE_Z: f32 = 1.08883;

impl From<LAB<f32>> for XYZ<f32> {
    fn from(value: LAB<f32>) -> Self {
        let lightness = value.l();
        let a_axis = value.ax();
        let b_axis = value.bx();

        let fy = (lightness + 16.0) / 116.0;
        let fx = a_axis / 500.0 + fy;
        let fz = fy - b_axis / 200.0;

        let yr = {
            let fy3 = fy.powf(3.);

            if fy3 > CIE_EPSILON {
                fy3
            } else {
                (116.0 * fy - 16.0) / CIE_KAPPA
            }
        };

        let xr = {
            let fx3 = fx.powf(3.0);

            if fx3 > CIE_EPSILON {
                fx3
            } else {
                (116.0 * fx - 16.0) / CIE_KAPPA
            }
        };

        let zr = {
            let fz3 = fz.powf(3.0);

            if fz3 > CIE_EPSILON {
                fz3
            } else {
                (116.0 * fz - 16.0) / CIE_KAPPA
            }
        };

        let x = xr * D65_WHITE_X;
        let y = yr * D65_WHITE_Y;
        let z = zr * D65_WHITE_Z;

        Self::new(x, y, z)
    }
}

impl From<XYZ<f32>> for LAB<f32> {
    fn from(value: XYZ<f32>) -> Self {
        let x = value.x();
        let y = value.y();
        let z = value.z();

        let xr = x / D65_WHITE_X;
        let yr = y / D65_WHITE_Y;
        let zr = z / D65_WHITE_Z;

        let fx = if xr > CIE_EPSILON {
            xr.cbrt()
        } else {
            (CIE_KAPPA * xr + 16.0) / 116.0
        };

        let fy = if yr > CIE_EPSILON {
            yr.cbrt()
        } else {
            (CIE_KAPPA * yr + 16.0) / 116.0
        };

        let fz = if yr > CIE_EPSILON {
            zr.cbrt()
        } else {
            (CIE_KAPPA * zr + 16.0) / 116.0
        };

        let lightness = 116.0 * fy - 16.0;
        let a_axis = 500.0 * (fx - fy);
        let b_axis = 200.0 * (fy - fz);

        Self::new(lightness, a_axis, b_axis)
    }
}

impl Mix for LAB<f32> {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let lightness = self.lightness.mix(target.lightness, t);
        let a_axis = self.a_axis.mix(target.a_axis, t);
        let b_axis = self.b_axis.mix(target.b_axis, t);

        Self::new(lightness, a_axis, b_axis)
    }
}
