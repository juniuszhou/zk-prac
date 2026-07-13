mod custom_field {
    #![allow(dead_code)]
    use ark_ec::short_weierstrass::{Affine, Projective, SWCurveConfig};
    use ark_ec::CurveConfig;
    use ark_ff::fields::{Fp64, MontBackend, MontConfig};
    use ark_ff::{Field, MontFp, Zero};
    #[modulus = "7"]
    #[generator = "3"]
    pub struct FqConfig;
    fn fqconfig___() {
        use ark_ff::{biginteger::arithmetic as fa, fields::Fp, fields::*, BigInt, BigInteger};
        type B = BigInt<1usize>;
        type F = Fp<MontBackend<FqConfig, 1usize>, 1usize>;
        #[automatically_derived]
        impl MontConfig<1usize> for FqConfig {
            const MODULUS: B = BigInt([7u64]);
            const GENERATOR: F = {
                let (is_positive, limbs) = (true, [3u64]);
                ::ark_ff::Fp::from_sign_and_limbs(is_positive, &limbs)
            };
            const TWO_ADIC_ROOT_OF_UNITY: F = {
                let (is_positive, limbs) = (true, [6u64]);
                ::ark_ff::Fp::from_sign_and_limbs(is_positive, &limbs)
            };
            #[inline(always)]
            fn add_assign(a: &mut F, b: &F) {
                __add_with_carry(&mut a.0, &b.0);
                __subtract_modulus(a);
            }
            #[inline(always)]
            fn sub_assign(a: &mut F, b: &F) {
                if b.0 > a.0 {
                    __add_with_carry(&mut a.0, &BigInt([7u64]));
                }
                __sub_with_borrow(&mut a.0, &b.0);
            }
            #[inline(always)]
            fn double_in_place(a: &mut F) {
                a.0.mul2();
                __subtract_modulus(a);
            }
            /// Sets `a = -a`.
            #[inline(always)]
            fn neg_in_place(a: &mut F) {
                if *a != F::ZERO {
                    let mut tmp = BigInt([7u64]);
                    __sub_with_borrow(&mut tmp, &a.0);
                    a.0 = tmp;
                }
            }
            #[inline(always)]
            fn mul_assign(a: &mut F, b: &F) {
                {
                    let mut r = [0u64; 1usize];
                    let mut carry1 = 0u64;
                    r[0] = fa::mac(r[0], (a.0).0[0], (b.0).0[0usize], &mut carry1);
                    let k = r[0].wrapping_mul(Self::INV);
                    let mut carry2 = 0u64;
                    fa::mac_discard(r[0], k, 7u64, &mut carry2);
                    r[1usize - 1] = carry1 + carry2;
                    (a.0).0 = r;
                }
                __subtract_modulus(a);
            }
            #[inline(always)]
            fn square_in_place(a: &mut F) {
                {
                    *a *= *a;
                }
            }
            fn sum_of_products<const M: usize>(a: &[F; M], b: &[F; M]) -> F {
                if M <= 121usize {
                    let result = (0..1usize).fold(BigInt::zero(), |mut result, j| {
                        let mut carry_a = 0;
                        let mut carry_b = 0;
                        for (a, b) in a.iter().zip(b) {
                            let a = &a.0;
                            let b = &b.0;
                            let mut carry2 = 0;
                            result.0[0] = fa::mac(result.0[0], a.0[j], b.0[0], &mut carry2);
                            carry_b = fa::adc(&mut carry_a, carry_b, carry2);
                        }
                        let k = result.0[0].wrapping_mul(Self::INV);
                        let mut carry2 = 0;
                        fa::mac_discard(result.0[0], k, 7u64, &mut carry2);
                        result.0[1usize - 1] = fa::adc_no_carry(carry_a, carry_b, &mut carry2);
                        result
                    });
                    let mut result = F::new_unchecked(result);
                    __subtract_modulus(&mut result);
                    if true {
                        match (&a.iter().zip(b).map(|(a, b)| *a * b).sum::<F>(), &result) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    let kind = ::core::panicking::AssertKind::Eq;
                                    ::core::panicking::assert_failed(
                                        kind,
                                        &*left_val,
                                        &*right_val,
                                        ::core::option::Option::None,
                                    );
                                }
                            }
                        };
                    }
                    result
                } else {
                    a.chunks(121usize)
                        .zip(b.chunks(121usize))
                        .map(|(a, b)| {
                            if a.len() == 121usize {
                                Self::sum_of_products::<121usize>(
                                    a.try_into().unwrap(),
                                    b.try_into().unwrap(),
                                )
                            } else {
                                a.iter().zip(b).map(|(a, b)| *a * b).sum()
                            }
                        })
                        .sum()
                }
            }
        }
        #[inline(always)]
        fn __subtract_modulus(a: &mut F) {
            if a.is_geq_modulus() {
                __sub_with_borrow(&mut a.0, &BigInt([7u64]));
            }
        }
        #[inline(always)]
        fn __subtract_modulus_with_carry(a: &mut F, carry: bool) {
            if a.is_geq_modulus() || carry {
                __sub_with_borrow(&mut a.0, &BigInt([7u64]));
            }
        }
        #[inline(always)]
        fn __add_with_carry(a: &mut B, b: &B) -> bool {
            use ark_ff::biginteger::arithmetic::adc_for_add_with_carry as adc;
            let mut carry = 0;
            carry = adc(&mut a.0[0usize], b.0[0usize], carry);
            carry != 0
        }
        #[inline(always)]
        fn __sub_with_borrow(a: &mut B, b: &B) -> bool {
            use ark_ff::biginteger::arithmetic::sbb_for_sub_with_borrow as sbb;
            let mut borrow = 0;
            borrow = sbb(&mut a.0[0usize], b.0[0usize], borrow);
            borrow != 0
        }
    }
    pub type Fq = Fp64<MontBackend<FqConfig, 1>>;
    #[modulus = "13"]
    #[generator = "2"]
    pub struct FrConfig;
    fn frconfig___() {
        use ark_ff::{biginteger::arithmetic as fa, fields::Fp, fields::*, BigInt, BigInteger};
        type B = BigInt<1usize>;
        type F = Fp<MontBackend<FrConfig, 1usize>, 1usize>;
        #[automatically_derived]
        impl MontConfig<1usize> for FrConfig {
            const MODULUS: B = BigInt([13u64]);
            const GENERATOR: F = {
                let (is_positive, limbs) = (true, [2u64]);
                ::ark_ff::Fp::from_sign_and_limbs(is_positive, &limbs)
            };
            const TWO_ADIC_ROOT_OF_UNITY: F = {
                let (is_positive, limbs) = (true, [8u64]);
                ::ark_ff::Fp::from_sign_and_limbs(is_positive, &limbs)
            };
            #[inline(always)]
            fn add_assign(a: &mut F, b: &F) {
                __add_with_carry(&mut a.0, &b.0);
                __subtract_modulus(a);
            }
            #[inline(always)]
            fn sub_assign(a: &mut F, b: &F) {
                if b.0 > a.0 {
                    __add_with_carry(&mut a.0, &BigInt([13u64]));
                }
                __sub_with_borrow(&mut a.0, &b.0);
            }
            #[inline(always)]
            fn double_in_place(a: &mut F) {
                a.0.mul2();
                __subtract_modulus(a);
            }
            /// Sets `a = -a`.
            #[inline(always)]
            fn neg_in_place(a: &mut F) {
                if *a != F::ZERO {
                    let mut tmp = BigInt([13u64]);
                    __sub_with_borrow(&mut tmp, &a.0);
                    a.0 = tmp;
                }
            }
            #[inline(always)]
            fn mul_assign(a: &mut F, b: &F) {
                {
                    let mut r = [0u64; 1usize];
                    let mut carry1 = 0u64;
                    r[0] = fa::mac(r[0], (a.0).0[0], (b.0).0[0usize], &mut carry1);
                    let k = r[0].wrapping_mul(Self::INV);
                    let mut carry2 = 0u64;
                    fa::mac_discard(r[0], k, 13u64, &mut carry2);
                    r[1usize - 1] = carry1 + carry2;
                    (a.0).0 = r;
                }
                __subtract_modulus(a);
            }
            #[inline(always)]
            fn square_in_place(a: &mut F) {
                {
                    *a *= *a;
                }
            }
            fn sum_of_products<const M: usize>(a: &[F; M], b: &[F; M]) -> F {
                if M <= 119usize {
                    let result = (0..1usize).fold(BigInt::zero(), |mut result, j| {
                        let mut carry_a = 0;
                        let mut carry_b = 0;
                        for (a, b) in a.iter().zip(b) {
                            let a = &a.0;
                            let b = &b.0;
                            let mut carry2 = 0;
                            result.0[0] = fa::mac(result.0[0], a.0[j], b.0[0], &mut carry2);
                            carry_b = fa::adc(&mut carry_a, carry_b, carry2);
                        }
                        let k = result.0[0].wrapping_mul(Self::INV);
                        let mut carry2 = 0;
                        fa::mac_discard(result.0[0], k, 13u64, &mut carry2);
                        result.0[1usize - 1] = fa::adc_no_carry(carry_a, carry_b, &mut carry2);
                        result
                    });
                    let mut result = F::new_unchecked(result);
                    __subtract_modulus(&mut result);
                    if true {
                        match (&a.iter().zip(b).map(|(a, b)| *a * b).sum::<F>(), &result) {
                            (left_val, right_val) => {
                                if !(*left_val == *right_val) {
                                    let kind = ::core::panicking::AssertKind::Eq;
                                    ::core::panicking::assert_failed(
                                        kind,
                                        &*left_val,
                                        &*right_val,
                                        ::core::option::Option::None,
                                    );
                                }
                            }
                        };
                    }
                    result
                } else {
                    a.chunks(119usize)
                        .zip(b.chunks(119usize))
                        .map(|(a, b)| {
                            if a.len() == 119usize {
                                Self::sum_of_products::<119usize>(
                                    a.try_into().unwrap(),
                                    b.try_into().unwrap(),
                                )
                            } else {
                                a.iter().zip(b).map(|(a, b)| *a * b).sum()
                            }
                        })
                        .sum()
                }
            }
        }
        #[inline(always)]
        fn __subtract_modulus(a: &mut F) {
            if a.is_geq_modulus() {
                __sub_with_borrow(&mut a.0, &BigInt([13u64]));
            }
        }
        #[inline(always)]
        fn __subtract_modulus_with_carry(a: &mut F, carry: bool) {
            if a.is_geq_modulus() || carry {
                __sub_with_borrow(&mut a.0, &BigInt([13u64]));
            }
        }
        #[inline(always)]
        fn __add_with_carry(a: &mut B, b: &B) -> bool {
            use ark_ff::biginteger::arithmetic::adc_for_add_with_carry as adc;
            let mut carry = 0;
            carry = adc(&mut a.0[0usize], b.0[0usize], carry);
            carry != 0
        }
        #[inline(always)]
        fn __sub_with_borrow(a: &mut B, b: &B) -> bool {
            use ark_ff::biginteger::arithmetic::sbb_for_sub_with_borrow as sbb;
            let mut borrow = 0;
            borrow = sbb(&mut a.0[0usize], b.0[0usize], borrow);
            borrow != 0
        }
    }
    pub type Fr = Fp64<MontBackend<FrConfig, 1>>;
    pub struct MyCurveConfig;
    #[automatically_derived]
    impl ::core::clone::Clone for MyCurveConfig {
        #[inline]
        fn clone(&self) -> MyCurveConfig {
            MyCurveConfig
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for MyCurveConfig {
        #[inline]
        fn default() -> MyCurveConfig {
            MyCurveConfig {}
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for MyCurveConfig {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for MyCurveConfig {
        #[inline]
        fn eq(&self, other: &MyCurveConfig) -> bool {
            true
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for MyCurveConfig {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    impl CurveConfig for MyCurveConfig {
        type BaseField = Fq;
        type ScalarField = Fr;
        const COFACTOR: &'static [u64] = &[1];
        const COFACTOR_INV: Fr = Fr::ONE;
    }
    impl SWCurveConfig for MyCurveConfig {
        const COEFF_A: Fq = Fq::ZERO;
        const COEFF_B: Fq = {
            let (is_positive, limbs) = (true, [3u64]);
            ::ark_ff::Fp::from_sign_and_limbs(is_positive, &limbs)
        };
        const GENERATOR: Affine<Self> = Affine::new_unchecked(G_X, G_Y);
        fn mul_by_a(_: Self::BaseField) -> Self::BaseField {
            Self::BaseField::zero()
        }
    }
    const G_X: Fq = {
        let (is_positive, limbs) = (true, [1u64]);
        ::ark_ff::Fp::from_sign_and_limbs(is_positive, &limbs)
    };
    const G_Y: Fq = {
        let (is_positive, limbs) = (true, [2u64]);
        ::ark_ff::Fp::from_sign_and_limbs(is_positive, &limbs)
    };
    pub type MyAffine = Affine<MyCurveConfig>;
    pub type MyProjective = Projective<MyCurveConfig>;
}
