use crate::{impl_bi_from_via, Alpha, Hsla, Hsva, Hwba, LinearRgba, Luminance, Mix, Oklaba, Srgba, StandardColor, Xyza};
use bevy_reflect::{Reflect, ReflectDeserialize, ReflectSerialize};
use serde::{Deserialize, Serialize};

/// Color in LAB color space, with alpha
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(PartialEq, Serialize, Deserialize)]
pub struct Laba {
    /// The lightness channel. [0.0, 1.5]
    pub lightness: f32,
    /// The a axis. [-1.5, 1.5]
    pub a: f32,
    /// The b axis. [-1.5, 1.5]
    pub b: f32,
    /// The alpha channel. [0.0, 1.0]
    pub alpha: f32,
}

impl StandardColor for Laba {}

impl Laba {
    /// Construct a new [`Laba`] color from components.
    ///
    /// # Arguments
    ///
    /// * `lightness` - Lightness channel. [0.0, 1.5]
    /// * `a` - a axis. [-1.5, 1.5]
    /// * `b` - b axis. [-1.5, 1.5]
    /// * `alpha` - Alpha channel. [0.0, 1.0]
    pub const fn new(lightness: f32, a: f32, b: f32, alpha: f32) -> Self {
        Self {
            lightness,
            a,
            b,
            alpha,
        }
    }

    /// Construct a new [`Laba`] color from (l, a, b) components, with the default alpha (1.0).
    ///
    /// # Arguments
    ///
    /// * `lightness` - Lightness channel. [0.0, 1.5]
    /// * `a` - a axis. [-1.5, 1.5]
    /// * `b` - b axis. [-1.5, 1.5]
    pub const fn lab(lightness: f32, a: f32, b: f32) -> Self {
        Self {
            lightness,
            a,
            b,
            alpha: 1.0,
        }
    }

    /// Return a copy of this color with the lightness channel set to the given value.
    pub const fn with_lightness(self, lightness: f32) -> Self {
        Self { lightness, ..self }
    }

    /// CIE Epsilon Constant
    ///
    /// See [Continuity (16) (17)](http://brucelindbloom.com/index.html?LContinuity.html)
    pub const CIE_EPSILON: f32 = 216.0 / 24389.0;

    /// CIE Kappa Constant
    ///
    /// See [Continuity (16) (17)](http://brucelindbloom.com/index.html?LContinuity.html)
    pub const CIE_KAPPA: f32 = 24389.0 / 27.0;
}

impl Default for Laba {
    fn default() -> Self {
        Self::new(1., 0., 0., 1.)
    }
}

impl Mix for Laba {
    #[inline]
    fn mix(&self, other: &Self, factor: f32) -> Self {
        let n_factor = 1.0 - factor;
        Self {
            lightness: self.lightness * n_factor + other.lightness * factor,
            a: self.a * n_factor + other.a * factor,
            b: self.b * n_factor + other.b * factor,
            alpha: self.alpha * n_factor + other.alpha * factor,
        }
    }
}

impl Alpha for Laba {
    #[inline]
    fn with_alpha(&self, alpha: f32) -> Self {
        Self { alpha, ..*self }
    }

    #[inline]
    fn alpha(&self) -> f32 {
        self.alpha
    }
}

impl Luminance for Laba {
    #[inline]
    fn with_luminance(&self, lightness: f32) -> Self {
        Self { lightness, ..*self }
    }

    fn luminance(&self) -> f32 {
        self.lightness
    }

    fn darker(&self, amount: f32) -> Self {
        Self::new(
            (self.lightness - amount).max(0.),
            self.a,
            self.b,
            self.alpha,
        )
    }

    fn lighter(&self, amount: f32) -> Self {
        Self::new(
            (self.lightness + amount).min(1.),
            self.a,
            self.b,
            self.alpha,
        )
    }
}

impl From<Laba> for Xyza {
    fn from(
        Laba {
            lightness,
            a,
            b,
            alpha,
        }: Laba,
    ) -> Self {
        // Based on http://www.brucelindbloom.com/index.html?Eqn_Lab_to_XYZ.html
        let l = 100. * lightness;
        let a = 100. * a;
        let b = 100. * b;

        let fy = (l + 16.0) / 116.0;
        let fx = a / 500.0 + fy;
        let fz = fy - b / 200.0;
        let xr = {
            let fx3 = fx.powf(3.0);

            if fx3 > Laba::CIE_EPSILON {
                fx3
            } else {
                (116.0 * fx - 16.0) / Laba::CIE_KAPPA
            }
        };
        let yr = if l > Laba::CIE_EPSILON * Laba::CIE_KAPPA {
            ((l + 16.0) / 116.0).powf(3.0)
        } else {
            l / Laba::CIE_KAPPA
        };
        let zr = {
            let fz3 = fz.powf(3.0);

            if fz3 > Laba::CIE_EPSILON {
                fz3
            } else {
                (116.0 * fz - 16.0) / Laba::CIE_KAPPA
            }
        };
        let x = xr * Xyza::D65_WHITE.x;
        let y = yr * Xyza::D65_WHITE.y;
        let z = zr * Xyza::D65_WHITE.z;

        Xyza::new(x, y, z, alpha)
    }
}

impl From<Xyza> for Laba {
    fn from(Xyza { x, y, z, alpha }: Xyza) -> Self {
        // Based on http://www.brucelindbloom.com/index.html?Eqn_XYZ_to_Lab.html
        let xr = x / Xyza::D65_WHITE.x;
        let yr = y / Xyza::D65_WHITE.y;
        let zr = z / Xyza::D65_WHITE.z;
        let fx = if xr > Laba::CIE_EPSILON {
            xr.cbrt()
        } else {
            (Laba::CIE_KAPPA * xr + 16.0) / 116.0
        };
        let fy = if yr > Laba::CIE_EPSILON {
            yr.cbrt()
        } else {
            (Laba::CIE_KAPPA * yr + 16.0) / 116.0
        };
        let fz = if yr > Laba::CIE_EPSILON {
            zr.cbrt()
        } else {
            (Laba::CIE_KAPPA * zr + 16.0) / 116.0
        };
        let l = 1.16 * fy - 0.16;
        let a = 5.00 * (fx - fy);
        let b = 2.00 * (fy - fz);

        Laba::new(l, a, b, alpha)
    }
}

impl_bi_from_via! {
    impl From<Srgba> for Laba via Xyza {}
    impl From<LinearRgba> for Laba via Xyza {}
    impl From<Hsla> for Laba via Xyza {}
    impl From<Hsva> for Laba via Xyza {}
    impl From<Hwba> for Laba via Xyza {}
    impl From<Oklaba> for Laba via Xyza {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        color_difference::EuclideanDistance, test_colors::TEST_COLORS, testing::assert_approx_eq,
        Srgba,
    };

    #[test]
    fn test_to_from_srgba() {
        for color in TEST_COLORS.iter() {
            let rgb2: Srgba = (color.lab).into();
            let laba: Laba = (color.rgb).into();
            assert!(
                color.rgb.distance(&rgb2) < 0.0001,
                "{}: {:?} != {:?}",
                color.name,
                color.rgb,
                rgb2
            );
            assert_approx_eq!(color.lab.lightness, laba.lightness, 0.001);
            if laba.lightness > 0.01 {
                assert_approx_eq!(color.lab.a, laba.a, 0.1);
            }
            if laba.lightness > 0.01 && laba.a > 0.01 {
                assert!(
                    (color.lab.b - laba.b).abs() < 1.7,
                    "{:?} != {:?}",
                    color.lab,
                    laba
                );
            }
            assert_approx_eq!(color.lab.alpha, laba.alpha, 0.001);
        }
    }

    #[test]
    fn test_to_from_linear() {
        for color in TEST_COLORS.iter() {
            let rgb2: LinearRgba = (color.lab).into();
            let laba: Laba = (color.linear_rgb).into();
            assert!(
                color.linear_rgb.distance(&rgb2) < 0.0001,
                "{}: {:?} != {:?}",
                color.name,
                color.linear_rgb,
                rgb2
            );
            assert_approx_eq!(color.lab.lightness, laba.lightness, 0.001);
            if laba.lightness > 0.01 {
                assert_approx_eq!(color.lab.a, laba.a, 0.1);
            }
            if laba.lightness > 0.01 && laba.a > 0.01 {
                assert!(
                    (color.lab.b - laba.b).abs() < 1.7,
                    "{:?} != {:?}",
                    color.lab,
                    laba
                );
            }
            assert_approx_eq!(color.lab.alpha, laba.alpha, 0.001);
        }
    }
}
