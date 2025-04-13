use crate::{
    ops::{self, FloatPow},
    Vec2, VectorSpace,
};
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::{std_traits::ReflectDefault, Reflect};

/// A segment of a cubic curve, used to hold precomputed coefficients for fast interpolation.
/// It is a [`Curve`] with domain `[0, 1]`.
///
/// Segments can be chained together to form a longer [compound curve].
///
/// [compound curve]: CubicCurve
/// [`Curve`]: crate::curve::Curve
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Clone)
)]
pub struct CubicSegment<P: VectorSpace> {
    /// Polynomial coefficients for the segment.
    pub coeff: [P; 4],
}

impl<P: VectorSpace> CubicSegment<P> {
    /// Instantaneous position of a point at parametric value `t`.
    #[inline]
    pub fn position(&self, t: f32) -> P {
        let [a, b, c, d] = self.coeff;
        // Evaluate `a + bt + ct^2 + dt^3`, avoiding exponentiation
        a + (b + (c + d * t) * t) * t
    }

    /// Instantaneous velocity of a point at parametric value `t`.
    #[inline]
    pub fn velocity(&self, t: f32) -> P {
        let [_, b, c, d] = self.coeff;
        // Evaluate the derivative, which is `b + 2ct + 3dt^2`, avoiding exponentiation
        b + (c * 2.0 + d * 3.0 * t) * t
    }

    /// Instantaneous acceleration of a point at parametric value `t`.
    #[inline]
    pub fn acceleration(&self, t: f32) -> P {
        let [_, _, c, d] = self.coeff;
        // Evaluate the second derivative, which is `2c + 6dt`
        c * 2.0 + d * 6.0 * t
    }

    /// Creates a cubic segment from four points, representing a Bezier curve.
    pub fn new_bezier(points: [P; 4]) -> Self {
        // A derivation for this matrix can be found in "General Matrix Representations for B-splines" by Kaihuai Qin.
        // <https://xiaoxingchen.github.io/2020/03/02/bspline_in_so3/general_matrix_representation_for_bsplines.pdf>
        // See section 4.2 and equation 11.
        let char_matrix = [
            [1., 0., 0., 0.],
            [-3., 3., 0., 0.],
            [3., -6., 3., 0.],
            [-1., 3., -3., 1.],
        ];
        Self::coefficients(points, char_matrix)
    }

    /// Calculate polynomial coefficients for the cubic curve using a characteristic matrix.
    #[inline]
    pub(super) fn coefficients(p: [P; 4], char_matrix: [[f32; 4]; 4]) -> Self {
        let [c0, c1, c2, c3] = char_matrix;
        // These are the polynomial coefficients, computed by multiplying the characteristic
        // matrix by the point matrix.
        let coeff = [
            p[0] * c0[0] + p[1] * c0[1] + p[2] * c0[2] + p[3] * c0[3],
            p[0] * c1[0] + p[1] * c1[1] + p[2] * c1[2] + p[3] * c1[3],
            p[0] * c2[0] + p[1] * c2[1] + p[2] * c2[2] + p[3] * c2[3],
            p[0] * c3[0] + p[1] * c3[1] + p[2] * c3[2] + p[3] * c3[3],
        ];
        Self { coeff }
    }

    /// A flexible iterator used to sample curves with arbitrary functions.
    ///
    /// This splits the curve into `subdivisions` of evenly spaced `t` values across the
    /// length of the curve from start (t = 0) to end (t = n), where `n = self.segment_count()`,
    /// returning an iterator evaluating the curve with the supplied `sample_function` at each `t`.
    ///
    /// For `subdivisions = 2`, this will split the curve into two lines, or three points, and
    /// return an iterator with 3 items, the three points, one at the start, middle, and end.
    #[inline]
    pub fn iter_samples<'a, 'b: 'a>(
        &'b self,
        subdivisions: usize,
        mut sample_function: impl FnMut(&Self, f32) -> P + 'a,
    ) -> impl Iterator<Item = P> + 'a {
        self.iter_uniformly(subdivisions)
            .map(move |t| sample_function(self, t))
    }

    /// An iterator that returns values of `t` uniformly spaced over `0..=subdivisions`.
    #[inline]
    fn iter_uniformly(&self, subdivisions: usize) -> impl Iterator<Item = f32> {
        let step = 1.0 / subdivisions as f32;
        (0..=subdivisions).map(move |i| i as f32 * step)
    }

    /// Iterate over the curve split into `subdivisions`, sampling the position at each step.
    pub fn iter_positions(&self, subdivisions: usize) -> impl Iterator<Item = P> + '_ {
        self.iter_samples(subdivisions, Self::position)
    }

    /// Iterate over the curve split into `subdivisions`, sampling the velocity at each step.
    pub fn iter_velocities(&self, subdivisions: usize) -> impl Iterator<Item = P> + '_ {
        self.iter_samples(subdivisions, Self::velocity)
    }

    /// Iterate over the curve split into `subdivisions`, sampling the acceleration at each step.
    pub fn iter_accelerations(&self, subdivisions: usize) -> impl Iterator<Item = P> + '_ {
        self.iter_samples(subdivisions, Self::acceleration)
    }
}

/// The `CubicSegment<Vec2>` can be used as a 2-dimensional easing curve for animation.
///
/// The x-axis of the curve is time, and the y-axis is the output value. This struct provides
/// methods for extremely fast solves for y given x.
impl CubicSegment<Vec2> {
    crate::cfg::alloc! {
        /// Construct a cubic Bezier curve for animation easing, with control points `p1` and `p2`. A
        /// cubic Bezier easing curve has control point `p0` at (0, 0) and `p3` at (1, 1), leaving only
        /// `p1` and `p2` as the remaining degrees of freedom. The first and last control points are
        /// fixed to ensure the animation begins at 0, and ends at 1.
        ///
        /// This is a very common tool for UI animations that accelerate and decelerate smoothly. For
        /// example, the ubiquitous "ease-in-out" is defined as `(0.25, 0.1), (0.25, 1.0)`.
        pub fn new_bezier_easing(p1: impl Into<Vec2>, p2: impl Into<Vec2>) -> Self {
            let (p0, p3) = (Vec2::ZERO, Vec2::ONE);
            Self::new_bezier([p0, p1.into(), p2.into(), p3])
        }
    }

    /// Maximum allowable error for iterative Bezier solve
    const MAX_ERROR: f32 = 1e-5;

    /// Maximum number of iterations during Bezier solve
    const MAX_ITERS: u8 = 8;

    /// Given a `time` within `0..=1`, returns an eased value that follows the cubic curve instead
    /// of a straight line. This eased result may be outside the range `0..=1`, however it will
    /// always start at 0 and end at 1: `ease(0) = 0` and `ease(1) = 1`.
    ///
    /// ```
    /// # use bevy_math::prelude::*;
    /// # bevy_math::cfg::alloc!
    /// # {
    /// let cubic_bezier = CubicSegment::new_bezier_easing((0.25, 0.1), (0.25, 1.0));
    /// assert_eq!(cubic_bezier.ease(0.0), 0.0);
    /// assert_eq!(cubic_bezier.ease(1.0), 1.0);
    /// # }
    /// ```
    ///
    /// # How cubic easing works
    ///
    /// Easing is generally accomplished with the help of "shaping functions". These are curves that
    /// start at (0,0) and end at (1,1). The x-axis of this plot is the current `time` of the
    /// animation, from 0 to 1. The y-axis is how far along the animation is, also from 0 to 1. You
    /// can imagine that if the shaping function is a straight line, there is a 1:1 mapping between
    /// the `time` and how far along your animation is. If the `time` = 0.5, the animation is
    /// halfway through. This is known as linear interpolation, and results in objects animating
    /// with a constant velocity, and no smooth acceleration or deceleration at the start or end.
    ///
    /// ```text
    /// y
    /// │         ●
    /// │       ⬈
    /// │     ⬈
    /// │   ⬈
    /// │ ⬈
    /// ●─────────── x (time)
    /// ```
    ///
    /// Using cubic Beziers, we have a curve that starts at (0,0), ends at (1,1), and follows a path
    /// determined by the two remaining control points (handles). These handles allow us to define a
    /// smooth curve. As `time` (x-axis) progresses, we now follow the curve, and use the `y` value
    /// to determine how far along the animation is.
    ///
    /// ```text
    /// y
    ///          ⬈➔●
    /// │      ⬈
    /// │     ↑
    /// │     ↑
    /// │    ⬈
    /// ●➔⬈───────── x (time)
    /// ```
    ///
    /// To accomplish this, we need to be able to find the position `y` on a curve, given the `x`
    /// value. Cubic curves are implicit parametric functions like B(t) = (x,y). To find `y`, we
    /// first solve for `t` that corresponds to the given `x` (`time`). We use the Newton-Raphson
    /// root-finding method to quickly find a value of `t` that is very near the desired value of
    /// `x`. Once we have this we can easily plug that `t` into our curve's `position` function, to
    /// find the `y` component, which is how far along our animation should be. In other words:
    ///
    /// > Given `time` in `0..=1`
    ///
    /// > Use Newton's method to find a value of `t` that results in B(t) = (x,y) where `x == time`
    ///
    /// > Once a solution is found, use the resulting `y` value as the final result
    #[inline]
    pub fn ease(&self, time: f32) -> f32 {
        let x = time.clamp(0.0, 1.0);
        self.find_y_given_x(x)
    }

    /// Find the `y` value of the curve at the given `x` value using the Newton-Raphson method.
    #[inline]
    fn find_y_given_x(&self, x: f32) -> f32 {
        let mut t_guess = x;
        let mut pos_guess = Vec2::ZERO;
        for _ in 0..Self::MAX_ITERS {
            pos_guess = self.position(t_guess);
            let error = pos_guess.x - x;
            if ops::abs(error) <= Self::MAX_ERROR {
                break;
            }
            // Using Newton's method, use the tangent line to estimate a better guess value.
            let slope = self.velocity(t_guess).x; // dx/dt
            t_guess -= error / slope;
        }
        pos_guess.y
    }
}

/// A segment of a rational cubic curve, used to hold precomputed coefficients for fast interpolation.
/// It is a [`Curve`] with domain `[0, 1]`.
///
/// Note that the `knot_span` is used only by [compound curves] constructed by chaining these
/// together.
///
/// [compound curves]: RationalCurve
/// [`Curve`]: crate::curve::Curve
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Clone)
)]
pub struct RationalSegment<P: VectorSpace> {
    /// The coefficients matrix of the cubic curve.
    pub coeff: [P; 4],
    /// The homogeneous weight coefficients.
    pub weight_coeff: [f32; 4],
    /// The width of the domain of this segment.
    pub knot_span: f32,
}
impl<P: VectorSpace> RationalSegment<P> {
    /// Instantaneous position of a point at parametric value `t` in `[0, 1]`.
    #[inline]
    pub fn position(&self, t: f32) -> P {
        let [a, b, c, d] = self.coeff;
        let [x, y, z, w] = self.weight_coeff;
        // Compute a cubic polynomial for the control points
        let numerator = a + (b + (c + d * t) * t) * t;
        // Compute a cubic polynomial for the weights
        let denominator = x + (y + (z + w * t) * t) * t;
        numerator / denominator
    }

    /// Instantaneous velocity of a point at parametric value `t` in `[0, 1]`.
    #[inline]
    pub fn velocity(&self, t: f32) -> P {
        // A derivation for the following equations can be found in "Matrix representation for NURBS
        // curves and surfaces" by Choi et al. See equation 19.

        let [a, b, c, d] = self.coeff;
        let [x, y, z, w] = self.weight_coeff;
        // Compute a cubic polynomial for the control points
        let numerator = a + (b + (c + d * t) * t) * t;
        // Compute a cubic polynomial for the weights
        let denominator = x + (y + (z + w * t) * t) * t;

        // Compute the derivative of the control point polynomial
        let numerator_derivative = b + (c * 2.0 + d * 3.0 * t) * t;
        // Compute the derivative of the weight polynomial
        let denominator_derivative = y + (z * 2.0 + w * 3.0 * t) * t;

        // Velocity is the first derivative (wrt to the parameter `t`)
        // Position = N/D therefore
        // Velocity = (N/D)' = N'/D - N * D'/D^2 = (N' * D - N * D')/D^2
        numerator_derivative / denominator
            - numerator * (denominator_derivative / denominator.squared())
    }

    /// Instantaneous acceleration of a point at parametric value `t` in `[0, 1]`.
    #[inline]
    pub fn acceleration(&self, t: f32) -> P {
        // A derivation for the following equations can be found in "Matrix representation for NURBS
        // curves and surfaces" by Choi et al. See equation 20. Note: In come copies of this paper, equation 20
        // is printed with the following two errors:
        // + The first term has incorrect sign.
        // + The second term uses R when it should use the first derivative.

        let [a, b, c, d] = self.coeff;
        let [x, y, z, w] = self.weight_coeff;
        // Compute a cubic polynomial for the control points
        let numerator = a + (b + (c + d * t) * t) * t;
        // Compute a cubic polynomial for the weights
        let denominator = x + (y + (z + w * t) * t) * t;

        // Compute the derivative of the control point polynomial
        let numerator_derivative = b + (c * 2.0 + d * 3.0 * t) * t;
        // Compute the derivative of the weight polynomial
        let denominator_derivative = y + (z * 2.0 + w * 3.0 * t) * t;

        // Compute the second derivative of the control point polynomial
        let numerator_second_derivative = c * 2.0 + d * 6.0 * t;
        // Compute the second derivative of the weight polynomial
        let denominator_second_derivative = z * 2.0 + w * 6.0 * t;

        // Velocity is the first derivative (wrt to the parameter `t`)
        // Position = N/D therefore
        // Velocity = (N/D)' = N'/D - N * D'/D^2 = (N' * D - N * D')/D^2
        // Acceleration = (N/D)'' = ((N' * D - N * D')/D^2)' = N''/D + N' * (-2D'/D^2) + N * (-D''/D^2 + 2D'^2/D^3)
        numerator_second_derivative / denominator
            + numerator_derivative * (-2.0 * denominator_derivative / denominator.squared())
            + numerator
                * (-denominator_second_derivative / denominator.squared()
                    + 2.0 * denominator_derivative.squared() / denominator.cubed())
    }

    /// Calculate polynomial coefficients for the cubic polynomials using a characteristic matrix.
    #[expect(clippy::allow_attributes, reason = "'alloc' feature is now un-named")]
    #[allow(
        dead_code,
        reason = "Method only used when `alloc` feature is enabled."
    )]
    #[inline]
    pub(super) fn coefficients(
        control_points: [P; 4],
        weights: [f32; 4],
        knot_span: f32,
        char_matrix: [[f32; 4]; 4],
    ) -> Self {
        // An explanation of this use can be found in "Matrix representation for NURBS curves and surfaces"
        // by Choi et al. See section "Evaluation of NURB Curves and Surfaces", and equation 16.

        let [c0, c1, c2, c3] = char_matrix;
        let p = control_points;
        let w = weights;
        // These are the control point polynomial coefficients, computed by multiplying the characteristic
        // matrix by the point matrix.
        let coeff = [
            p[0] * c0[0] + p[1] * c0[1] + p[2] * c0[2] + p[3] * c0[3],
            p[0] * c1[0] + p[1] * c1[1] + p[2] * c1[2] + p[3] * c1[3],
            p[0] * c2[0] + p[1] * c2[1] + p[2] * c2[2] + p[3] * c2[3],
            p[0] * c3[0] + p[1] * c3[1] + p[2] * c3[2] + p[3] * c3[3],
        ];
        // These are the weight polynomial coefficients, computed by multiplying the characteristic
        // matrix by the weight matrix.
        let weight_coeff = [
            w[0] * c0[0] + w[1] * c0[1] + w[2] * c0[2] + w[3] * c0[3],
            w[0] * c1[0] + w[1] * c1[1] + w[2] * c1[2] + w[3] * c1[3],
            w[0] * c2[0] + w[1] * c2[1] + w[2] * c2[2] + w[3] * c2[3],
            w[0] * c3[0] + w[1] * c3[1] + w[2] * c3[2] + w[3] * c3[3],
        ];
        Self {
            coeff,
            weight_coeff,
            knot_span,
        }
    }
}

impl<P: VectorSpace> From<CubicSegment<P>> for RationalSegment<P> {
    fn from(value: CubicSegment<P>) -> Self {
        Self {
            coeff: value.coeff,
            weight_coeff: [1.0, 0.0, 0.0, 0.0],
            knot_span: 1.0, // Cubic curves are uniform, so every segment has domain [0, 1).
        }
    }
}
