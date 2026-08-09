//! Mini KZG — 从零实现 KZG 多项式承诺的核心操作
//!
//! KZG 基于配对椭圆曲线，核心思想:
//! - Setup: 选择随机 τ (trapdoor)，公开 g^{τⁱ} (G1) 和 h^τ (G2)
//! - Commit: C = g^{p(τ)}，用已知的 g^{τⁱ} 和系数 a_i 计算
//! - Prove:  π = g^{q(τ)}，其中 q(x) = (p(x)-p(z))/(x-z)
//! - Verify: e(C - g^{p(z)}, h) == e(π, h^τ - h^z)
//!
//! 安全性: τ 必须在 setup 后被丢弃，否则可以伪造证明。

use ark_bn254::{Bn254, Fr, G1Projective, G2Projective};
use ark_ec::pairing::Pairing;
use ark_ec::Group;
use ark_ff::{UniformRand, Zero};
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use rand::Rng;

// ============================================================================
// KZG 数据结构
// ============================================================================

/// CRS (公共参考串) — KZG setup 的输出
struct KZGCRS {
    /// g^{τⁱ} for i=0..degree = [g, g^τ, g^{τ²}, ...]
    powers_of_g: Vec<G1Projective>,
    /// h^τ, G2 上的元素
    h_tau: G2Projective,
    /// h (G2 生成元)
    h: G2Projective,
    /// g (G1 生成元)
    g: G1Projective,
}

/// 多项式承诺 (就是一个 G1 点: C = g^{p(τ)})
#[derive(Debug)]
struct Commitment(G1Projective);

/// 求值证明 (也是一个 G1 点: π = g^{q(τ)})
#[derive(Debug)]
struct Proof(G1Projective);

// ============================================================================
// KZG Setup — 可信设置
// ============================================================================
//
// 1. 随机选择 τ ∈ Fr (trapdoor)，设置完成后必须丢弃
// 2. 计算 g^{τⁱ} for i=0..max_degree (G1)
//    这允许对任意度数 ≤ max_degree 的多项式做承诺
// 3. 计算 h^τ (G2)
// 4. CRS = (powers_of_g, h^τ, g, h)

fn setup(max_degree: usize, rng: &mut impl Rng) -> KZGCRS {
    // 随机选取 trapdoor τ
    let tau = Fr::rand(rng);

    let g = G1Projective::generator();
    let h = G2Projective::generator();

    // 计算 g^{τⁱ} for i=0..max_degree
    // 方法: 不断乘以 τ 比每次重新计算快
    let mut powers_of_g = Vec::with_capacity(max_degree);
    let mut current = g;
    powers_of_g.push(current); // g^{τ⁰} = g
    for _ in 1..max_degree {
        current = current * tau; // current = current · τ = g^{τⁱ}
        powers_of_g.push(current);
    }

    // h^τ
    let h_tau = h * tau;

    KZGCRS {
        powers_of_g,
        h_tau,
        h,
        g,
    }
}

// ============================================================================
// KZG Commit — 对多项式 p(x) 做承诺
// ============================================================================
//
// C = g^{p(τ)} = Σ a_i · g^{τⁱ}  (multi-scalar multiplication)
// 其中 a_i 是 p(x) = Σ a_i xⁱ 的系数

fn commit(crs: &KZGCRS, poly: &DensePolynomial<Fr>) -> Commitment {
    let coeffs = &poly.coeffs;
    // MSM: Σ a_i · powers_of_g[i]
    let mut result = G1Projective::zero();
    for (i, coeff) in coeffs.iter().enumerate() {
        // 检查是否超出 CRS 支持的最大度数
        if i >= crs.powers_of_g.len() {
            panic!("多项式度数超出 CRS 支持的范围");
        }
        // the value of polynomial
        result += crs.powers_of_g[i] * coeff;
    }
    Commitment(result)
}

// ============================================================================
// KZG Prove — 证明 p(z) = v . choose z according to challenge rule and give v as result
// ============================================================================
//
// 1. 合成除法计算 q(x) = (p(x) - v) / (x - z)
//    使用 Horner 方法，从最高次系数开始
// 2. π = g^{q(τ)} = commit(q)
// Proof is a point in G1, same as Commitment
fn prove(crs: &KZGCRS, poly: &DensePolynomial<Fr>, z: Fr, v: Fr) -> Proof {
    // 多项式除法: (p(x) - v) / (x - z) = q(x)
    let q_poly: DensePolynomial<Fr> = divide_by_linear(poly, z, v);
    Proof(commit(crs, &q_poly).0)
}

/// 合成除法: 计算 (p(x) - v) / (x - z)
///
/// 从最高次系数 a_n 开始:
///   b_{n-1} = a_n
///   b_{i-1} = a_i + z·b_i  for i = n-1, ..., 1
/// 余数 r = a_0 + z·b_0 应该等于 p(z) = v, 证明除法精确
fn divide_by_linear(poly: &DensePolynomial<Fr>, z: Fr, v: Fr) -> DensePolynomial<Fr> {
    let coeffs = &poly.coeffs;
    let n = coeffs.len();
    let mut q_coeffs = Vec::with_capacity(n - 1);
    let mut carry: Fr = coeffs[n - 1]; // a_n
    for i in (1..n).rev() {
        q_coeffs.push(carry); // b_{i-1} = carry
        carry = coeffs[i - 1] + carry * z; // carry = a_{i-1} + z·b_{i-1}
    }
    // carry 现在是 p(z), 应等于 v
    debug_assert_eq!(carry, v, "合成除法余数应等于 p(z)");
    q_coeffs.reverse();
    DensePolynomial::from_coefficients_vec(q_coeffs)
}

// ============================================================================
// KZG Verify — 验证求值证明
// ============================================================================
//
// 验证方程: e(C - g^v, h) == e(π, h^τ - h^z)
//
// 推导:
//   p(τ) - v = q(τ)·(τ - z)           (多项式除法定义)
//   g^{p(τ)-v} = g^{q(τ)·(τ-z)} = π^{τ-z}
//   e(g^{p(τ)-v}, h) = e(π^{τ-z}, h)
//   e(C/g^v, h) = e(π, h^{τ-z})
//   e(C - g^v, h) = e(π, h^τ - h^z)   (双线性)

fn verify(crs: &KZGCRS, comm: &Commitment, z: Fr, v: Fr, proof: &Proof) -> bool {
    // C - g^v  (G1)
    let gv = crs.g * v;
    let c_minus_gv = comm.0 - gv;

    // h^τ - h^z  (G2)
    let hz = crs.h * z;
    let h_tau_minus_hz = crs.h_tau - hz;

    // e(C - g^v, h) == e(π, h^τ - h^z)
    let lhs = Bn254::pairing(c_minus_gv, crs.h);
    let rhs = Bn254::pairing(proof.0, h_tau_minus_hz);

    lhs == rhs
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_kzg_lifecycle() {
        let mut rng = StdRng::seed_from_u64(42);

        // ── Setup: 支持度数 ≤ 8 ──────────────────────────────────────
        let crs = setup(8, &mut rng);
        assert_eq!(crs.powers_of_g.len(), 8);

        // ── 多项式: p(x) = 2x³ + 3x² + 4x + 5, degree = 3 ────────────
        let poly = DensePolynomial::from_coefficients_slice(&[
            Fr::from(5u64), // 常数项
            Fr::from(4u64), // x
            Fr::from(3u64), // x²
            Fr::from(2u64), // x³
        ]);
        assert_eq!(poly.degree(), 3);

        // ── Commit ────────────────────────────────────────────────────
        let comm = commit(&crs, &poly);

        // ── 在 z = 7 处求值并生成证明 ────────────────────────────────
        let z = Fr::from(7u64);
        let v = poly.evaluate(&z);
        // p(7) = 2·343 + 3·49 + 4·7 + 5 = 686 + 147 + 28 + 5 = 866
        assert_eq!(v, Fr::from(866u64));

        let proof = prove(&crs, &poly, z, v);

        // ── Verify ────────────────────────────────────────────────────
        let result = verify(&crs, &comm, z, v, &proof);
        assert!(result, "KZG 验证应该通过");

        // ── 错误值/伪造证明应该验证失败 ──────────────────────────
        let wrong_v = Fr::from(999u64);

        // 正确证明 + 错误值 → 失败
        let result = verify(&crs, &comm, z, wrong_v, &proof);
        assert!(!result, "错误的值与正确的证明不匹配");

        // 伪造证明 + 正确值 → 失败
        let fake_proof = Proof(G1Projective::rand(&mut rng));
        let result = verify(&crs, &comm, z, v, &fake_proof);
        assert!(!result, "伪造证明与正确的值不匹配");

        // 伪造证明 + 错误值 → 失败
        let result = verify(&crs, &comm, z, wrong_v, &fake_proof);
        assert!(!result, "伪造证明与错误的值也不匹配");
    }
}
