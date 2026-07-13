#![allow(dead_code)]

use ark_ec::short_weierstrass::{Affine, Projective, SWCurveConfig};
use ark_ec::CurveConfig;
use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::{Field, MontFp, Zero};

// ── Base field Fq: F_7 (curve coordinates live here) ────────────────
#[derive(MontConfig)]
#[modulus = "7"]
#[generator = "3"]
pub struct FqConfig;

pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

// ── Scalar field Fr: F_13 (curve order = 13, prime) ─────────────────
#[derive(MontConfig)]
#[modulus = "13"]
#[generator = "2"]
pub struct FrConfig;
pub type Fr = Fp64<MontBackend<FrConfig, 1>>;

// ── Short Weierstrass curve: y² = x³ + 3 over F_7 ───────────────────
// The curve has 13 points (prime order), so Fr = F_13.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct MyCurveConfig;

impl CurveConfig for MyCurveConfig {
    type BaseField = Fq;
    type ScalarField = Fr;
    const COFACTOR: &'static [u64] = &[1];
    const COFACTOR_INV: Fr = Fr::ONE;
}

impl SWCurveConfig for MyCurveConfig {
    const COEFF_A: Fq = Fq::ZERO;
    const COEFF_B: Fq = MontFp!("3");
    const GENERATOR: Affine<Self> = Affine::new_unchecked(G_X, G_Y);

    fn mul_by_a(_: Self::BaseField) -> Self::BaseField {
        Self::BaseField::zero()
    }
}

// Generator point coordinates
const G_X: Fq = MontFp!("1");
const G_Y: Fq = MontFp!("2");

pub type MyAffine = Affine<MyCurveConfig>;
pub type MyProjective = Projective<MyCurveConfig>;
