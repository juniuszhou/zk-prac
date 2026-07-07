use ark_ec::pairing::Pairing;
use ark_ec::{CurveGroup, Group};
use ark_ff::{Field, PrimeField};
use ark_std::{One, UniformRand, Zero};
use rand::rngs::StdRng;
use rand::SeedableRng;

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
