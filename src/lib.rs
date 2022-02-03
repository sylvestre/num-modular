//! This crate provides efficient Modular arithmetic operations
//! for various integer types, including primitive integers and
//! `num-bigint`. The latter option is enabled optionally.

use std::ops::{Add, Mul, Neg, Sub};

/// This trait describes modular arithmetic operations
pub trait ModularOps<Rhs = Self, Modulus = Self> {
    type Output;

    /// Return (self + rhs) % m
    fn addm(self, rhs: Rhs, m: Modulus) -> Self::Output;

    /// Return (self + rhs) % m
    fn subm(self, rhs: Rhs, m: Modulus) -> Self::Output;

    /// Return (self * rhs) % m
    fn mulm(self, rhs: Rhs, m: Modulus) -> Self::Output;

    /// Return (self ^ exp) % m
    fn powm(self, exp: Rhs, m: Modulus) -> Self::Output;

    /// Return (-self) % m and make sure the result is normalized in range [0,m)
    fn negm(self, m: Modulus) -> Self::Output;

    /// Calculate modular inverse (x such that self*x = 1 mod m).
    ///
    /// This operation is only available for integer that is coprime to `m`
    fn invm(self, m: Modulus) -> Option<Self::Output>
    where
        Self: Sized;

    /// Calculate Jacobi Symbol (a|n), where a is self
    ///
    /// Note that we don't provide Legendre symbol function
    /// here, as it depends on primality test. However, if
    /// n is surely a prime, this function can be directly used as
    /// Legendre symbol.
    ///
    /// # Panics
    /// if n is negative or even
    fn jacobi(self, n: Modulus) -> i8;

    /// Calculate Kronecker Symbol (a|n), where a is self
    fn kronecker(self, n: Modulus) -> i8;

    // TODO: ModularOps sqrt aka Quadratic residue
    // fn sqrtm(self, m: Modulus);
}

/// Represents an number defined in a modulo ring ℤ/nℤ
///
/// The operators should panic if the modulus of two number
/// are not the same.
pub trait ModularInteger:
    Sized
    + PartialEq
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Neg<Output = Self>
    + Mul<Self, Output = Self>
    // TODO: impl Pow
{
    /// The underlying representation type of the integer
    type Base;

    /// Return the modulus of the ring
    fn modulus(&self) -> &Self::Base;

    /// Return the normalized residue of this integer in the ring
    fn residue(&self) -> Self::Base;

    /// Convert an normal integer into the same ring.
    ///
    /// This method should be perferred over the static
    /// constructor to prevent unnecessary overhead of pre-computation.
    fn new(&self, n: Self::Base) -> Self;
}

mod monty;
mod prim;
pub use monty::{Montgomery, MontgomeryInt};

#[cfg(feature = "num-bigint")]
mod bigint;

// tests for ModularOps goes here
#[cfg(test)]
mod tests {
    use super::*;
    use rand;

    #[cfg(feature = "num-bigint")]
    use num_bigint::BigUint;

    #[test]
    fn u64_basic_mod_test() {
        let a = rand::random::<u64>() % 100000;
        let m = rand::random::<u64>() % 100000;
        assert_eq!(a.addm(a, &m), (a + a) % m);
        assert_eq!(a.mulm(a, &m), (a * a) % m);
        assert_eq!(a.powm(3, &m), a.pow(3) % m);
    }

    #[test]
    #[cfg(feature = "num-bigint")]
    fn biguint_basic_mod_test() {
        let a = rand::random::<u128>();
        let ra = &BigUint::from(a);
        let m = rand::random::<u128>();
        let rm = &BigUint::from(m);
        assert_eq!(ra.addm(ra, rm), (ra + ra) % rm);
        assert_eq!(ra.mulm(ra, rm), (ra * ra) % rm);
        assert_eq!(ra.powm(BigUint::from(3u8), rm), ra.pow(3) % rm);
    }

    #[test]
    fn monty_int_basic_test() {
        let a = rand::random::<u8>();
        let m = rand::random::<u8>();
        let m = m >> m.trailing_zeros();
        assert_eq!(MontgomeryInt::new(a, m).residue(), a % m);

        let a = rand::random::<u16>();
        let m = rand::random::<u16>();
        let m = m >> m.trailing_zeros();
        assert_eq!(MontgomeryInt::new(a, m).residue(), a % m);

        let a = rand::random::<u32>();
        let m = rand::random::<u32>();
        let m = m >> m.trailing_zeros();
        assert_eq!(MontgomeryInt::new(a, m).residue(), a % m);

        let a = rand::random::<u64>();
        let m = rand::random::<u64>();
        let m = m >> m.trailing_zeros();
        assert_eq!(MontgomeryInt::new(a, m).residue(), a % m);
    }

    const ADDM_CASES: [(u8, u8, u8, u8); 10] = [
        // [m, x, y, rem]: x + y = rem (mod m)
        (5, 0, 0, 0),
        (5, 1, 2, 3),
        (5, 2, 1, 3),
        (5, 2, 2, 4),
        (5, 3, 2, 0),
        (5, 2, 3, 0),
        (5, 6, 1, 2),
        (5, 1, 6, 2),
        (5, 11, 7, 3),
        (5, 7, 11, 3),
    ];

    #[test]
    fn addm_test() {
        for (m, x, y, r) in ADDM_CASES.iter() {
            assert_eq!(x.addm(y, &m), *r, "u8 x: {}, y: {}", x, y);
            assert_eq!((*x as u16).addm(*y as u16, &(*m as u16)), *r as u16);
            assert_eq!((*x as u32).addm(*y as u32, &(*m as u32)), *r as u32);
            assert_eq!((*x as u64).addm(*y as u64, &(*m as u64)), *r as u64);
            assert_eq!((*x as u128).addm(*y as u128, &(*m as u128)), *r as u128);

            #[cfg(feature = "num-bigint")]
            {
                assert_eq!(
                    BigUint::from(*x).addm(BigUint::from(*y), &BigUint::from(*m)),
                    BigUint::from(*r)
                );
            }
        }
    }

    #[test]
    fn monty_add_test() {
        for (m, x, y, r) in ADDM_CASES.iter() {
            let mx = MontgomeryInt::new(*x, *m as u8);
            let my = MontgomeryInt::new(*y, *m as u8);
            assert_eq!((mx + my).residue(), *r);

            // test the `new()` method
            let mx = MontgomeryInt::new(*x, *m as u8);
            let my = mx.new(*y);
            assert_eq!((mx + my).residue(), *r);

            let mx = MontgomeryInt::new(*x as u16, *m as u16);
            let my = MontgomeryInt::new(*y as u16, *m as u16);
            assert_eq!((mx + my).residue(), *r as u16);

            let mx = MontgomeryInt::new(*x as u32, *m as u32);
            let my = MontgomeryInt::new(*y as u32, *m as u32);
            assert_eq!((mx + my).residue(), *r as u32);

            let mx = MontgomeryInt::new(*x as u64, *m as u64);
            let my = MontgomeryInt::new(*y as u64, *m as u64);
            assert_eq!((mx + my).residue(), *r as u64);
        }
    }

    const SUBM_CASES: [(u8, u8, u8, u8); 10] = [
        // [m, x, y, rem]: x - y = rem (mod m)
        (7, 0, 0, 0),
        (7, 11, 9, 2),
        (7, 5, 2, 3),
        (7, 2, 5, 4),
        (7, 6, 7, 6),
        (7, 1, 7, 1),
        (7, 7, 1, 6),
        (7, 0, 6, 1),
        (7, 15, 1, 0),
        (7, 1, 15, 0),
    ];

    #[test]
    fn subm_test() {
        for (m, x, y, r) in SUBM_CASES.iter() {
            assert_eq!(x.subm(y, &m), *r);
            assert_eq!((*x as u16).subm(*y as u16, &(*m as u16)), *r as u16);
            assert_eq!((*x as u32).subm(*y as u32, &(*m as u32)), *r as u32);
            assert_eq!((*x as u64).subm(*y as u64, &(*m as u64)), *r as u64);
            assert_eq!((*x as u128).subm(*y as u128, &(*m as u128)), *r as u128);

            #[cfg(feature = "num-bigint")]
            {
                assert_eq!(
                    BigUint::from(*x).subm(BigUint::from(*y), &BigUint::from(*m)),
                    BigUint::from(*r),
                );
            }
        }
    }

    // TODO: add test for mul and pow

    #[test]
    fn monty_sub_test() {
        for (m, x, y, r) in SUBM_CASES.iter() {
            let mx = MontgomeryInt::new(*x, *m as u8);
            let my = MontgomeryInt::new(*y, *m as u8);
            assert_eq!((mx - my).residue(), *r);

            // test the `new()` method
            let mx = MontgomeryInt::new(*x, *m as u8);
            let my = mx.new(*y);
            assert_eq!((mx - my).residue(), *r);

            let mx = MontgomeryInt::new(*x as u16, *m as u16);
            let my = MontgomeryInt::new(*y as u16, *m as u16);
            assert_eq!((mx - my).residue(), *r as u16);

            let mx = MontgomeryInt::new(*x as u32, *m as u32);
            let my = MontgomeryInt::new(*y as u32, *m as u32);
            assert_eq!((mx - my).residue(), *r as u32);

            let mx = MontgomeryInt::new(*x as u64, *m as u64);
            let my = MontgomeryInt::new(*y as u64, *m as u64);
            assert_eq!((mx - my).residue(), *r as u64);
        }
    }

    #[test]
    fn invm_test() {
        let test_cases: [(u64, u64, u64); 8] = [
            // [a, m, x] s.t. a*x = 1 (mod m) is satisfied
            (5, 11, 9),
            (8, 11, 7),
            (10, 11, 10),
            (3, 5000, 1667),
            (1667, 5000, 3),
            (999, 5000, 3999),
            (999, 9_223_372_036_854_775_807, 3_619_181_019_466_538_655),
            (
                9_223_372_036_854_775_804,
                9_223_372_036_854_775_807,
                3_074_457_345_618_258_602,
            ),
        ];

        for (a, m, x) in test_cases.iter() {
            assert_eq!(ModularOps::<&u64>::invm(a, m).unwrap(), *x);

            #[cfg(feature = "num-bigint")]
            {
                assert_eq!(
                    ModularOps::<&BigUint>::invm(&BigUint::from(*a), &BigUint::from(*m)).unwrap(),
                    BigUint::from(*x)
                );
            }
        }
    }

    #[test]
    fn jacobi_test() {
        let test_cases: [(u8, u8, i8); 15] = [
            (1, 1, 1),
            (15, 1, 1),
            (2, 3, -1),
            (29, 9, 1),
            (4, 11, 1),
            (17, 11, -1),
            (19, 29, -1),
            (10, 33, -1),
            (11, 33, 0),
            (12, 33, 0),
            (14, 33, -1),
            (15, 33, 0),
            (15, 37, -1),
            (29, 59, 1),
            (30, 59, -1),
        ];

        for (a, n, res) in test_cases.iter() {
            assert_eq!(ModularOps::<&u8>::jacobi(a, n), *res);
            assert_eq!(ModularOps::<&u16>::jacobi(&(*a as u16), &(*n as u16)), *res);
            assert_eq!(ModularOps::<&u32>::jacobi(&(*a as u32), &(*n as u32)), *res);
            assert_eq!(ModularOps::<&u64>::jacobi(&(*a as u64), &(*n as u64)), *res);
            assert_eq!(
                ModularOps::<&u128>::jacobi(&(*a as u128), &(*n as u128)),
                *res
            );

            #[cfg(feature = "num-bigint")]
            {
                assert_eq!(
                    ModularOps::<&BigUint>::jacobi(&(BigUint::from(*a)), &(BigUint::from(*n))),
                    *res
                );
            }
        }
    }

    #[test]
    fn kronecker_test() {
        let test_cases: [(u8, u8, i8); 18] = [
            (0, 15, 0),
            (1, 15, 1),
            (2, 15, 1),
            (4, 15, 1),
            (7, 15, -1),
            (10, 15, 0),
            (0, 14, 0),
            (1, 14, 1),
            (2, 14, 0),
            (4, 14, 0),
            (9, 14, 1),
            (10, 14, 0),
            (0, 11, 0),
            (1, 11, 1),
            (2, 11, -1),
            (4, 11, 1),
            (9, 11, 1),
            (10, 11, -1),
        ];

        for (a, n, res) in test_cases.iter() {
            assert_eq!(ModularOps::<&u8>::kronecker(a, n), *res);
            assert_eq!(
                ModularOps::<&u16>::kronecker(&(*a as u16), &(*n as u16)),
                *res
            );
            assert_eq!(
                ModularOps::<&u32>::kronecker(&(*a as u32), &(*n as u32)),
                *res
            );
            assert_eq!(
                ModularOps::<&u64>::kronecker(&(*a as u64), &(*n as u64)),
                *res
            );
            assert_eq!(
                ModularOps::<&u128>::kronecker(&(*a as u128), &(*n as u128)),
                *res
            );

            #[cfg(feature = "num-bigint")]
            {
                assert_eq!(
                    ModularOps::<&BigUint>::kronecker(&(BigUint::from(*a)), &(BigUint::from(*n))),
                    *res
                );
            }
        }
    }
}
