use ark_ec::pairing::Pairing;
use ark_ec::{CurveGroup, Group};
use ark_ff::{FftField, Field, PrimeField};
use ark_std::{One, UniformRand, Zero};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[test]
fn test_basic_bn254() {
    let a: ark_bn254::Fr = ark_bn254::Fr::from(2u64);
    let b: ark_bn254::Fr = ark_bn254::Fr::from(3u64);
    let c = a + b;

    let d: ark_bn254::Fq = ark_bn254::Fq::from(2u64);
    assert_eq!(c, ark_bn254::Fr::from(5u64));

    // can not compare Fr and Fq directly
    // assert_eq!(a, d);

    // they are just value, so we can compare their bigints
    assert_eq!(a.into_bigint(), d.into_bigint());

    let g1 = <ark_bn254::G1Projective>::generator();
    let p: ark_bn254::G1Projective = g1 * <ark_bn254::Fr>::from(5u64);
    // (x y z) structure
    println!("p = {:?}", p);
    // x and y in Fq, or (x, y) in affine coordinates
    // (x, y) is a point on the curve of elliptic curve, and z is the projective coordinate
    // curve equation: y² = x³ + ax + b, where a and b are curve parameters
    // it is y² = x³ + 3, so a = 0, b = 3 for BN254 curve
    println!("p as affine = {:?}", p.into_affine());

    let q = p.into_affine();
    println!("q.x() = {:?}", type_of(&q.x));
    println!("q.x() = {:?}", q.x.into_bigint());
    println!("q.y() = {:?}", q.y.into_bigint());
}

// ─── BN254 ───────────────────────────────────────────────────────────────

#[test]
fn test_bn254() {
    let mut rng = StdRng::seed_from_u64(42);

    // 1. Field info
    println!("── test_bn254 ──");
    println!(
        "  Fr (scalar field)  bit-width: {} bits",
        ark_bn254::Fr::MODULUS_BIT_SIZE
    );
    println!(
        "  Fq (base field)    bit-width: {} bits",
        ark_bn254::Fq::MODULUS_BIT_SIZE
    );

    // 2. Fr arithmetic
    let a: ark_bn254::Fr = UniformRand::rand(&mut rng);
    let b: ark_bn254::Fr = UniformRand::rand(&mut rng);
    let a_inv = a.inverse().unwrap();
    assert_eq!(a * a_inv, <ark_bn254::Fr>::one());
    assert!(((a + b) * (a - b) - (a.square() - b.square())).is_zero());
    println!("  ✓ Scalar field (Fr) arithmetic");

    // 3. G1 operations
    let g1 = <ark_bn254::G1Projective>::generator();
    assert!(!g1.is_zero());
    let g1_aff: ark_bn254::G1Affine = g1.into();
    assert!(g1_aff.is_on_curve());
    assert!(g1_aff.is_in_correct_subgroup_assuming_on_curve());
    let p: ark_bn254::G1Projective = g1 * <ark_bn254::Fr>::from(5u64);
    let sum: ark_bn254::G1Projective =
        (g1 * <ark_bn254::Fr>::from(2u64)) + (g1 * <ark_bn254::Fr>::from(3u64));
    assert_eq!(sum.into_affine(), p.into_affine());
    println!("  ✓ G1 (generator + scalar mul + addition)");

    // 4. G2 operations
    let g2 = <ark_bn254::G2Projective>::generator();
    assert!(!g2.is_zero());
    let g2_aff: ark_bn254::G2Affine = g2.into();
    assert!(g2_aff.is_on_curve());
    assert!(g2_aff.is_in_correct_subgroup_assuming_on_curve());
    let q: ark_bn254::G2Projective = g2 * <ark_bn254::Fr>::from(7u64);
    assert!(q.into_affine().is_on_curve());
    println!("  ✓ G2 (generator + scalar mul)");

    // 5. Pairing: e : G1 × G2 → GT
    let gt = <ark_bn254::Bn254>::pairing(g1, g2);
    assert!(!gt.is_zero(), "Non-degeneracy: e(G,H) ≠ 1");

    let aa: ark_bn254::Fr = <ark_bn254::Fr>::from(2u64);
    let bb: ark_bn254::Fr = <ark_bn254::Fr>::from(3u64);
    let left = <ark_bn254::Bn254>::pairing(g1 * aa, g2 * bb);
    let right = gt * (aa * bb);
    assert_eq!(left, right, "Bilinearity");

    let multi = <ark_bn254::Bn254 as Pairing>::multi_pairing(
        [g1, g1 * <ark_bn254::Fr>::from(2u64)],
        [g2, g2],
    );
    let expected = <ark_bn254::Bn254>::pairing(g1 * <ark_bn254::Fr>::from(3u64), g2);
    assert_eq!(multi, expected);
    println!("  ✓ Pairing (bilinearity + non-degeneracy + multi-pairing)");
    println!();
}

// ─── BLS12-381 ───────────────────────────────────────────────────────────

#[test]
fn test_bls12_381() {
    let mut rng = StdRng::seed_from_u64(42);

    // 1. Field info
    println!("── test_bls12_381 ──");
    println!(
        "  Fr (scalar field)  bit-width: {} bits",
        ark_bls12_381::Fr::MODULUS_BIT_SIZE
    );
    println!(
        "  Fq (base field)    bit-width: {} bits",
        ark_bls12_381::Fq::MODULUS_BIT_SIZE
    );

    // 2. Fr arithmetic
    let a: ark_bls12_381::Fr = UniformRand::rand(&mut rng);
    let b: ark_bls12_381::Fr = UniformRand::rand(&mut rng);
    let a_inv = a.inverse().unwrap();
    assert_eq!(a * a_inv, <ark_bls12_381::Fr>::one());
    assert!(((a + b) * (a - b) - (a.square() - b.square())).is_zero());
    println!("  ✓ Scalar field (Fr) arithmetic");

    // 3. G1 operations
    let g1 = <ark_bls12_381::G1Projective>::generator();
    assert!(!g1.is_zero());
    let g1_aff: ark_bls12_381::G1Affine = g1.into();

    assert!(g1_aff.is_on_curve());
    assert!(g1_aff.is_in_correct_subgroup_assuming_on_curve());
    let p: ark_bls12_381::G1Projective = g1 * <ark_bls12_381::Fr>::from(5u64);
    let sum: ark_bls12_381::G1Projective =
        (g1 * <ark_bls12_381::Fr>::from(2u64)) + (g1 * <ark_bls12_381::Fr>::from(3u64));
    assert_eq!(sum.into_affine(), p.into_affine());
    println!("  ✓ G1 (generator + scalar mul + addition)");

    // 4. G2 operations
    let g2 = <ark_bls12_381::G2Projective>::generator();
    assert!(!g2.is_zero());
    let g2_aff: ark_bls12_381::G2Affine = g2.into();
    assert!(g2_aff.is_on_curve());
    assert!(g2_aff.is_in_correct_subgroup_assuming_on_curve());
    let q: ark_bls12_381::G2Projective = g2 * <ark_bls12_381::Fr>::from(7u64);
    assert!(q.into_affine().is_on_curve());
    println!("  ✓ G2 (generator + scalar mul)");

    // 5. Pairing: e : G1 × G2 → GT
    let gt = <ark_bls12_381::Bls12_381>::pairing(g1, g2);
    assert!(!gt.is_zero(), "Non-degeneracy: e(G,H) ≠ 1");

    let aa: ark_bls12_381::Fr = <ark_bls12_381::Fr>::from(2u64);
    let bb: ark_bls12_381::Fr = <ark_bls12_381::Fr>::from(3u64);
    let left = <ark_bls12_381::Bls12_381>::pairing(g1 * aa, g2 * bb);
    let right = gt * (aa * bb);
    assert_eq!(left, right, "Bilinearity");

    let multi = <ark_bls12_381::Bls12_381 as Pairing>::multi_pairing(
        [g1, g1 * <ark_bls12_381::Fr>::from(2u64)],
        [g2, g2],
    );
    let expected = <ark_bls12_381::Bls12_381>::pairing(g1 * <ark_bls12_381::Fr>::from(3u64), g2);
    assert_eq!(multi, expected);
    println!("  ✓ Pairing (bilinearity + non-degeneracy + multi-pairing)");
    println!();
}

#[test]
fn test_fq_usage() {
    let mut rng = StdRng::seed_from_u64(42);
    println!("── test_fq_usage (BLS12-381 Fq) ──");

    // 1. Fq: the base field — coordinates of G1 live here
    type Fq = ark_bls12_381::Fq;
    type Fr = ark_bls12_381::Fr;
    type G1 = ark_bls12_381::G1Projective;

    let a: Fq = UniformRand::rand(&mut rng);
    let b: Fq = UniformRand::rand(&mut rng);
    println!("  a = {a}");
    println!("  b = {b}");

    // Arithmetic — same Field trait
    assert_eq!(a + Fq::zero(), a);
    assert_eq!(a * Fq::one(), a);
    let _sum = a + b; // works: a + b ∈ Fq
    let _prod = a * b; // works: a * b ∈ Fq
    let inv = a.inverse().unwrap();
    assert_eq!(a * inv, Fq::one());
    assert_eq!(
        (a + b).square(),
        a.square() + Fq::from(2u64) * a * b + b.square()
    );
    println!("  ✓ +, -, *, /, square");

    // 2. Fq vs Fr — totally different primes
    println!(
        "  Fq bit-width = {} bits, Fr bit-width = {} bits",
        Fq::MODULUS_BIT_SIZE,
        Fr::MODULUS_BIT_SIZE
    );
    assert_ne!(
        Fq::MODULUS_BIT_SIZE,
        Fr::MODULUS_BIT_SIZE,
        "Fq and Fr are different fields"
    );

    // 3. Fq can NOT be used as scalar for G1 — only Fr works
    let g = G1::generator();
    // g * a  ← type error: can't multiply by Fq

    // But Fq *IS* the coordinate type for G1 affine points
    let g_aff: ark_bls12_381::G1Affine = g.into_affine();
    let _x: Fq = g_aff.x; // x coordinate in Fq
    let _y: Fq = g_aff.y; // y coordinate in Fq
    println!("  ✓ G1Affine stores coordinates as Fq");

    // 4. Fq has 2-adicity for FFT (over the *base* field)
    let two_adic = <Fq as FftField>::TWO_ADICITY;
    assert!(two_adic > 0);
    let root = <Fq as FftField>::TWO_ADIC_ROOT_OF_UNITY;
    assert!(root.pow(&[1u64 << two_adic]) == Fq::one());
    println!("  ✓ Fq 2-adicity = {two_adic}, root-of-unity works");

    // 5. Construct Fq from integer / bytes
    let five = Fq::from(5u64);
    let from_bytes = Fq::from_le_bytes_mod_order(&5u64.to_le_bytes());
    assert_eq!(five, from_bytes);

    // 6. Legendre symbol / quadratic residue
    // In Fq, half the elements are squares (QR), half are non-squares (QNR)
    let _legendre = a.legendre(); // returns LegendreSymbol
    let a_sqrt = a.sqrt(); // Some if QR, None if QNR
    if let Some(s) = a_sqrt {
        assert_eq!(s.square(), a, "sqrt² = a");
    }
    println!("  ✓ legendre() and sqrt()");
}
