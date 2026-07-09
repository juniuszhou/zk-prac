use ark_bls12_381::{Bls12_381, Fr};
use ark_poly::univariate::DensePolynomial;
use ark_poly::{DenseUVPolynomial, Polynomial};
use ark_poly_commit::kzg10::{
    Commitment, Powers, Proof, Randomness, UniversalParams, VerifierKey, KZG10,
};
use ark_std::test_rng;
use std::borrow::Cow;

type UniPoly = DensePolynomial<Fr>;
type KZG = KZG10<Bls12_381, UniPoly>;

#[test]
fn test_poly_commit() {
    let rng = &mut test_rng();

    // ===== Step 1: Setup (Trusted Setup) =====
    // 生成 CRS = (g, g^τ, g^τ², ..., g^τ^d, h, h^τ)
    // τ (trapdoor) 是随机数，设置后必须丢弃
    // max_degree: 支持的最大多项式度数
    // false: 不生成 G2 的幂次（只生成 G1 的幂次即可）
    let max_degree: usize = 8;
    let pp: UniversalParams<Bls12_381> =
        KZG::setup(max_degree, false, rng).expect("KZG setup failed");

    // ===== Step 2: Trim — 提取当前度数需要的密钥材料 =====
    // 从 UniversalParams 中截取 degree≤4 的部分
    // Powers: 给证明者——结构包含 g^{τ^i} 和 g^{γ·τ^i}
    // VerifierKey: 给验证者——结构包含 g, h, h^τ (β = τ)
    let supported_degree: usize = 4;
    let powers_of_g: Vec<ark_bls12_381::G1Affine> = pp.powers_of_g[..=supported_degree].to_vec();
    let powers_of_gamma_g: Vec<ark_bls12_381::G1Affine> = (0..=supported_degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();
    let powers: Powers<'_, Bls12_381> = Powers {
        powers_of_g: Cow::Owned(powers_of_g),
        powers_of_gamma_g: Cow::Owned(powers_of_gamma_g),
    };
    let vk: VerifierKey<Bls12_381> = VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };

    // ===== Step 3: 创建多项式 p(x) =====
    // p(x) = 3x³ + 2x² + x + 7
    let coeffs: Vec<Fr> = vec![7u64.into(), 1u64.into(), 2u64.into(), 3u64.into()];

    let poly: UniPoly = UniPoly::from_coefficients_vec(coeffs);
    println!("poly: {:?}", poly);
    assert_eq!(poly.degree(), 3);

    // ===== Step 4: Commit — 生成多项式承诺 =====
    // C = g^{p(τ)} = ∏ (g^{τ^i})^{a_i}
    // 其中 a_i 是多项式 p 的系数
    // hiding_bound=None: 不隐藏多项式（仅绑定，无零知识）
    let (comm, randomness): (Commitment<Bls12_381>, Randomness<Fr, UniPoly>) = KZG::commit(
        &powers,
        &poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .expect("KZG commit failed");

    println!("comm: {:?}", comm);

    // ===== Step 5: 在随机点 z 处求值 =====
    let z: Fr = 5u64.into();
    let v: Fr = poly.evaluate(&z);
    // p(5) = 3·125 + 2·25 + 5 + 7 = 375 + 50 + 5 + 7 = 437
    println!("v = p(z) = {:?}", v);

    // ===== Step 6: Prove — 生成求值证明 =====
    // KZG 证明的核心代数结构——商多项式:
    //   商多项式 q(x) = (p(x) - v) / (x - z)
    //   由于 x=z 是 p(x)-v 的根，多项式除法得到精确的 q(x)
    //
    // 证明 π = g^{q(τ)} = commit(q(x))
    // 验证者通过配对检查: e(C / g^v, h) = e(π, h^τ / h^z)
    //                 等价于 e(C - g^v, h) = e(π, h^{τ-z})
    let (witness_poly, _): (UniPoly, Option<UniPoly>) =
        KZG::compute_witness_polynomial(&poly, z, &randomness).expect("compute witness failed");

    println!("witness_poly: {:?}", witness_poly);

    // commit is a point in G1, i.e., g^{q(τ)}, including a G1 element and a G2 element (for hiding)
    let (comm_w, _): (Commitment<Bls12_381>, Randomness<Fr, UniPoly>) = KZG::commit(
        &powers,
        &witness_poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .expect("KZG commit witness failed");

    let proof: Proof<Bls12_381> = Proof {
        w: comm_w.0,
        random_v: None,
    };
    // actual proof is the w, which is a point in G1, i.e., g^{q(τ)}
    println!("proof: {:?}", proof.w);

    // ===== Step 7: Verify — 验证求值证明 =====
    // 验证方程: e(C - g^v, h) = e(π, h^β - h^z)
    // 其中 β = τ (trapdoor)，h 是 G2 生成元
    //
    // 推导:
    //   C = g^{p(τ)}, π = g^{q(τ)}
    //   q(τ) = (p(τ) - v) / (τ - z)
    //   ⇒ p(τ) - v = q(τ) · (τ - z)
    //   ⇒ g^{p(τ)-v} = g^{q(τ)·(τ-z)} = π^{τ-z}
    //   ⇒ 配对 e(g^{p(τ)-v}, h) = e(π, h^{τ-z})
    let ok: bool = KZG::check(&vk, &comm, z, v, &proof).expect("KZG check failed");

    assert!(ok, "KZG proof verification must succeed");
}
