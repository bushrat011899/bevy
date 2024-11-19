//! Provides implementations for methods relating to the [Greatest Common Divisor](https://en.wikipedia.org/wiki/Greatest_common_divisor).

/// Computes the [Greatest Common Divisor](https://en.wikipedia.org/wiki/Greatest_common_divisor)
/// between the supplied `value` and the constant parameter `N`.
///
/// This method uses the [Euclidean Algorithm](https://en.wikipedia.org/wiki/Euclidean_algorithm#Implementations).
/// Specifically, the division-based implementation proposed by D.E. Knuth in _The Art of Computer Programming_.
#[inline(always)]
pub const fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Computes the [Greatest Common Divisor](https://en.wikipedia.org/wiki/Greatest_common_divisor)
/// between the supplied `value` and the constant parameter `N`.
///
/// This method uses a lookup table computed at compile-time, avoiding branches.
/// If the second value `N` is not a constant knowable at compile-time, or it's
/// impractically large, consider using [`gcd`] instead.
///
/// This method does _not_ panic.
#[inline(always)]
pub const fn gcd_by_table<const N: usize>(value: u64) -> u64 {
    const fn gcd_table<const N: usize>() -> [u64; N] {
        let mut table: [u64; N] = [1; N];
        let a = N as u64;
        let mut b = 0;

        while b < a {
            table[b as usize] = gcd(a, b);

            b += 1;
        }

        table
    }

    if N == 0 {
        return 0;
    }

    let table = const {
        // Taking a reference avoids copying this table
        &gcd_table::<N>()
    };

    table[(value % N as u64) as usize]
}

/// Computes `N / gcd(N, value)`, where `gcd(N, value)` is the [Greatest Common Divisor](https://en.wikipedia.org/wiki/Greatest_common_divisor)
/// between the supplied `value` and the constant parameter `N`.
///
/// This method uses a lookup table computed at compile-time, avoiding branches.
/// If the second value `N` is not a constant knowable at compile-time, or it's
/// impractically large, consider using [`gcd`] directly instead.
///
/// This method does _not_ panic.
///
/// For the case where `N` is zero, the returned value is zero, instead of a runtime panic.
#[inline(always)]
pub const fn n_over_gcd_by_table<const N: usize>(value: u64) -> u64 {
    const fn n_over_gcd_table<const N: usize>() -> [u64; N] {
        let mut table: [u64; N] = [1; N];
        let a = N as u64;
        let mut b = 0;

        while b < a {
            table[b as usize] = a / gcd(a, b);

            b += 1;
        }

        table
    }

    if N == 0 {
        return 0;
    }

    let table = const {
        // Taking a reference avoids copying this table
        &n_over_gcd_table::<N>()
    };

    table[(value % N as u64) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_tests() {
        assert_eq!(gcd(0, 0), 0);

        assert_eq!(gcd(1, 0), 1);
        assert_eq!(gcd(1, 1), 1);

        assert_eq!(gcd(2, 0), 2);
        assert_eq!(gcd(2, 1), 1);
        assert_eq!(gcd(2, 2), 2);

        assert_eq!(gcd(3, 0), 3);
        assert_eq!(gcd(3, 1), 1);
        assert_eq!(gcd(3, 2), 1);
        assert_eq!(gcd(3, 3), 3);

        assert_eq!(gcd(4, 0), 4);
        assert_eq!(gcd(4, 1), 1);
        assert_eq!(gcd(4, 2), 2);
        assert_eq!(gcd(4, 3), 1);
        assert_eq!(gcd(4, 4), 4);
    }

    #[test]
    fn gcd_tests_symmetry() {
        for a in 0..1000 {
            for b in 0..1000 {
                assert_eq!(gcd(a, b), gcd(b, a));
            }
        }
    }

    #[test]
    fn gcd_by_table_tests() {
        /// We compare [`gcd_by_table`] to [`gcd`], as [`gcd`] is tested above.
        const fn gcd_by_table_test<const N: usize>() {
            let mut i = 0;
            while i <= N {
                assert!(gcd_by_table::<N>(i as u64) == gcd(N as u64, i as u64));
                i += 1;
            }
        }

        gcd_by_table_test::<0>();
        gcd_by_table_test::<1>();
        gcd_by_table_test::<2>();
        gcd_by_table_test::<3>();
        gcd_by_table_test::<4>();
        gcd_by_table_test::<5>();
        gcd_by_table_test::<6>();
        gcd_by_table_test::<7>();
        gcd_by_table_test::<8>();
    }

    #[test]
    fn n_over_gcd_by_table_tests() {
        /// We compare [`n_over_gcd_by_table`] to [`gcd`], as [`gcd`] is tested above.
        const fn gcd_by_table_test<const N: usize>() {
            let mut i = 0;
            while i <= 2 * N {
                assert!(n_over_gcd_by_table::<N>(i as u64) == N as u64 / gcd(N as u64, i as u64));
                i += 1;
            }
        }

        gcd_by_table_test::<1>();
        gcd_by_table_test::<2>();
        gcd_by_table_test::<3>();
        gcd_by_table_test::<4>();
        gcd_by_table_test::<5>();
        gcd_by_table_test::<6>();
        gcd_by_table_test::<7>();
        gcd_by_table_test::<8>();
    }
}
