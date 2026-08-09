// The custom F_7 / F_13 fields defined in src/custom_field.rs
// (mod 7 arithmetic is what the asserts below rely on).
use ark_ec::{CurveGroup, Group};
use ark_ff::{FftField, Field, One, PrimeField, Zero};
use ark_std::UniformRand;
use arkworks::custom_field::{Fq, Fr, MyProjective};
use rand::rngs::StdRng;
use rand::SeedableRng;
#[test]
fn test_fq_arithmetic() {
    let a = Fq::from(3_u64);
    let b = Fq::from(5_u64);

    assert_eq!(a + b, Fq::from(1_u64)); // 8 ≡ 1 mod 7
    assert_eq!(a - b, Fq::from(5_u64)); // -2 ≡ 5 mod 7
    assert_eq!(a * b, Fq::from(1_u64)); // 15 ≡ 1 mod 7
    assert_eq!(a.square(), Fq::from(2_u64)); // 9 ≡ 2 mod 7

    let inv = a.inverse().unwrap();
    assert_eq!(a * inv, Fq::one());
}

#[test]
fn test_fq_properties() {
    assert_eq!(Fq::MODULUS_BIT_SIZE, 3);

    // FftField::GENERATOR = full multiplicative group generator (= primitive root)
    assert_eq!(Fq::GENERATOR, Fq::from(3_u64));

    let one = Fq::one();
    assert!(!one.is_zero());
    assert!(Fq::zero().is_zero());

    let a = Fq::from(7_u64);
    assert_eq!(a, Fq::zero()); // 7 ≡ 0 mod 7
}

#[test]
fn test_fq_multiplicative_group() {
    let g = Fq::from(3_u64);
    let mut cur = Fq::one();
    let mut seen = Vec::new();
    for _ in 0..6 {
        cur *= g;
        seen.push(cur.into_bigint().0[0]);
    }
    // 3 generates F_7* : powers are 3, 2, 6, 4, 5, 1
    assert_eq!(seen, vec![3, 2, 6, 4, 5, 1]);
    assert_eq!(cur, Fq::one());
}

#[test]
fn test_fr_arithmetic() {
    let a = Fr::from(10_u64);
    let b = Fr::from(7_u64);

    assert_eq!(a + b, Fr::from(4_u64)); // 17 ≡ 4 mod 13
    assert_eq!(a - b, Fr::from(3_u64)); // 3 ≡ 3 mod 13
    assert_eq!(a * b, Fr::from(5_u64)); // 70 ≡ 5 mod 13
    assert_eq!(a.square(), Fr::from(9_u64)); // 100 ≡ 9 mod 13

    let inv = a.inverse().unwrap();
    assert_eq!(a * inv, Fr::one());

    assert!(Fr::zero().is_zero());
    assert!(!Fr::one().is_zero());
}

#[test]
fn test_fr_properties() {
    assert_eq!(Fr::MODULUS_BIT_SIZE, 4);

    // Fr::GENERATOR = full multiplicative group generator (= primitive root)
    assert_eq!(Fr::GENERATOR, Fr::from(2_u64));
}

#[test]
fn test_generator_on_curve() {
    let g = MyProjective::generator();
    assert!(!g.is_zero());

    let g_aff = g.into_affine();
    assert!(g_aff.is_on_curve());
    assert!(g_aff.is_in_correct_subgroup_assuming_on_curve());
}

#[test]
fn test_scalar_mul_identity() {
    // G * 13 = identity (since curve order is 13)
    let g = MyProjective::generator();
    let identity = g * Fr::from(13_u64);
    assert!(identity.is_zero());

    // G * 0 = identity
    let zero_scalar = g * Fr::zero();
    assert!(zero_scalar.is_zero());

    // G * 1 = G
    let one_scalar = g * Fr::one();
    assert_eq!(one_scalar, g);
}

#[test]
fn test_point_addition() {
    let g = MyProjective::generator();

    let g2 = g.double();
    let g3 = g * Fr::from(3_u64);
    let g5 = g * Fr::from(5_u64);

    // 2G + 3G = 5G
    assert_eq!(g2 + g3, g5);

    // 2G = G + G
    assert_eq!(g + g, g2);
}

#[test]
fn test_point_negation() {
    let g = MyProjective::generator();
    let g_neg = -g;
    assert_eq!(g + g_neg, MyProjective::zero());
}

#[test]
fn test_random_scalar_mul() {
    let mut rng = StdRng::seed_from_u64(42);
    let g = MyProjective::generator();

    let a: Fr = UniformRand::rand(&mut rng);
    let b: Fr = UniformRand::rand(&mut rng);

    let p = g * a;
    let q = g * b;
    let r = g * (a + b);

    // (a + b)G = aG + bG
    assert_eq!(p + q, r);

    // (ab)G = a(bG)
    let s = g * (a * b);
    let t = q * a;
    assert_eq!(s, t);
}

#[test]
fn test_pairing_fq_fr_independence() {
    // Fq and Fr are different fields (mod 7 vs mod 13)
    assert_ne!(Fq::MODULUS_BIT_SIZE, Fr::MODULUS_BIT_SIZE);

    let a_fq = Fq::from(5_u64);
    let a_fr = Fr::from(5_u64);

    // Same integer, but different field — bigints are the same
    assert_eq!(a_fq.into_bigint(), a_fr.into_bigint());

    // Fq(5) + Fq(5) = Fq(3) mod 7, Fr(5) + Fr(5) = Fr(10) mod 13
    assert_eq!(a_fq + a_fq, Fq::from(3_u64));
    assert_eq!(a_fr + a_fr, Fr::from(10_u64));
}
