//! FRI 承诺方案: 将多项式评估值建成 Merkle 树, 然后通过折叠协议生成证明
//!
//! 流程:
//! 1. 将多项式 p(x) 在 n 个域点上求值 → Merkle 树 → 根作为承诺
//! 2. 重复折叠: p_{r+1}(y) = p_e(y) + β·p_o(y), 度减半, 构建下一棵 Merkle 树
//! 3. 最终得到常数 → 作为证明的一部分
//! 4. 验证: 检查某一行在每层打开的值是否满足折叠关系 + Merkle 证明

use ark_bn254::Fr;
use ark_ff::{BigInteger, FftField, Field, One, PrimeField, UniformRand, Zero};
use ark_poly::{
    univariate::DensePolynomial, DenseUVPolynomial, EvaluationDomain, Polynomial,
    Radix2EvaluationDomain,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use sha2::{Digest, Sha256};

// ============================================================================
// 哈希工具
// ============================================================================

fn fr_to_bytes(e: &Fr) -> Vec<u8> {
    e.into_bigint().to_bytes_be()
}

fn hash_fr(e: &Fr) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(&fr_to_bytes(e));
    let r = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&r);
    out
}

fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    // 排序保证确定性
    if a < b {
        h.update(a);
        h.update(b);
    } else {
        h.update(b);
        h.update(a);
    }
    let r = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&r);
    out
}

// ============================================================================
// Merkle 树
// ============================================================================

struct MerkleTree {
    leaves: Vec<Fr>,
    /// layers[0] = 叶子哈希层, layers[last] = 根
    layers: Vec<Vec<[u8; 32]>>,
}

fn build_merkle(leaves: &[Fr]) -> MerkleTree {
    assert!(leaves.len().is_power_of_two());
    let mut layers: Vec<Vec<[u8; 32]>> = vec![leaves.iter().map(hash_fr).collect()];
    while layers.last().unwrap().len() > 1 {
        let prev = layers.last().unwrap();
        layers.push(prev.chunks(2).map(|c| hash_pair(&c[0], &c[1])).collect());
    }
    MerkleTree {
        leaves: leaves.to_vec(),
        layers,
    }
}

fn merkle_root(tree: &MerkleTree) -> &[u8; 32] {
    &tree.layers.last().unwrap()[0]
}

fn merkle_proof(tree: &MerkleTree, mut idx: usize) -> Vec<[u8; 32]> {
    let mut path = Vec::new();
    for layer in &tree.layers[..tree.layers.len() - 1] {
        path.push(layer[if idx % 2 == 0 { idx + 1 } else { idx - 1 }]);
        idx /= 2;
    }
    path
}

fn verify_merkle(root: &[u8; 32], leaf_hash: &[u8; 32], path: &[[u8; 32]], mut idx: usize) -> bool {
    let mut cur = *leaf_hash;
    for sibling in path {
        cur = if idx % 2 == 0 {
            hash_pair(&cur, sibling)
        } else {
            hash_pair(sibling, &cur)
        };
        idx /= 2;
    }
    &cur == root
}

// ============================================================================
// FRI 核心
// ============================================================================

/// 将多项式系数按奇偶拆分并折叠: c'_i = c_{2i} + β·c_{2i+1}
fn fold_poly(poly: &DensePolynomial<Fr>, beta: Fr) -> DensePolynomial<Fr> {
    let coeffs = &poly.coeffs;
    let mut padded = coeffs.clone();
    if padded.len() % 2 != 0 {
        padded.push(Fr::zero());
    }
    let folded: Vec<Fr> = padded.chunks(2).map(|c| c[0] + beta * c[1]).collect();
    DensePolynomial::from_coefficients_slice(&folded)
}

struct FriProof {
    /// 每轮的承诺: (Merkle 根, β)
    rounds: Vec<([u8; 32], Fr)>,
    /// 最终常数多项式的值
    final_constant: Fr,

    /// 单查询打开: 所有层的 (左值, 右值, 左 Merkle 路径, 右 Merkle 路径, 右叶子索引)
    openings: Vec<(Fr, Fr, Vec<[u8; 32]>, Vec<[u8; 32]>, usize)>,
    query_index: usize,
}

/// FRI 协议完整执行: 承诺 + 折叠 + 生成打开证明
fn fri_prove(
    poly: &DensePolynomial<Fr>,
    domain: &Radix2EvaluationDomain<Fr>,
    rng: &mut StdRng,
) -> (FriProof, Vec<MerkleTree>) {
    let n = domain.size();
    let mut trees: Vec<MerkleTree> = Vec::new();
    let mut rounds: Vec<([u8; 32], Fr)> = Vec::new();
    let mut cur_poly = poly.clone();
    let mut cur_domain = *domain;
    let mut cur_size = n;

    // ---- 折叠: 在每层评估 → Merkle → 折半多项式 ----
    // 当多项式已经是常数 (degree = 0) 时停止折叠
    while cur_size > 1 && cur_poly.degree() > 0 {
        let evals: Vec<Fr> = cur_domain
            .elements()
            .map(|x| cur_poly.evaluate(&x))
            .collect();
        let tree = build_merkle(&evals);
        trees.push(tree);
        let beta = Fr::rand(rng);
        rounds.push((*merkle_root(trees.last().unwrap()), beta));
        cur_poly = fold_poly(&cur_poly, beta);
        cur_size /= 2;
        cur_domain = Radix2EvaluationDomain::new(cur_size).unwrap();
    }

    // 最终层: degree-0, 常数
    let final_constant = cur_poly.evaluate(&Fr::one());

    // ---- 为查询索引 i=0 生成打开路径 ----
    let qi = 0usize;
    let mut openings = Vec::new();
    for (r, tree) in trees.iter().enumerate() {
        let size_r = n >> r; // 当前层叶子数
        let left = tree.leaves[qi];
        let right = tree.leaves[qi + size_r / 2];
        let lp = merkle_proof(tree, qi);
        let rp = merkle_proof(tree, qi + size_r / 2);
        openings.push((left, right, lp, rp, qi + size_r / 2));
    }

    (
        FriProof {
            rounds,
            final_constant,
            openings,
            query_index: qi,
        },
        trees,
    )
}

/// 验证 FRI 证明
fn fri_verify(proof: &FriProof, domain: &Radix2EvaluationDomain<Fr>) -> bool {
    let n = domain.size();

    // 1. 验证每层的折叠一致性
    for r in 0..proof.rounds.len() {
        let size_half = (n >> r) / 2; // n_r / 2
        if proof.query_index >= size_half {
            break;
        } // 超出范围就停

        let (left, right) = (proof.openings[r].0, proof.openings[r].1);
        let beta = proof.rounds[r].1;

        // 域元素 g = ω^{2ʳ·qi}
        let omega = Fr::get_root_of_unity(n as u64).unwrap();
        let g = omega.pow(&[(proof.query_index as u64) << r]);

        let two_inv = Fr::from(2u64).inverse().unwrap();
        let even = (left + right) * two_inv;
        let odd = (left - right) * two_inv * g.inverse().unwrap();
        let folded = even + beta * odd;

        let ok = if r + 1 < proof.rounds.len() {
            folded == *&proof.openings[r + 1].0
        } else {
            folded == proof.final_constant
        };

        if !ok {
            let expected = if r + 1 < proof.rounds.len() {
                proof.openings[r + 1].0
            } else {
                proof.final_constant
            };
            println!("  ❌ 第 {r} 轮折叠检查失败");
            println!(
                "     left={left:?}, right={right:?}, folded={folded:?}, expected={expected:?}"
            );
            return false;
        }
    }

    // 2. 验证 Merkle 证明
    for r in 0..proof.rounds.len() {
        let root = &proof.rounds[r].0;
        let (left, right, ref lp, ref rp, right_idx) = proof.openings[r];
        let lh = hash_fr(&left);
        let rh = hash_fr(&right);

        if !verify_merkle(root, &lh, lp, proof.query_index) {
            println!("  ❌ 第 {r} 轮左叶子 Merkle 证明失败");
            return false;
        }
        if !verify_merkle(root, &rh, rp, right_idx) {
            println!("  ❌ 第 {r} 轮右叶子 Merkle 证明失败 (idx={right_idx})");
            return false;
        }
    }

    true
}

// ============================================================================
// Demo
// ============================================================================

pub fn run_fri_demo() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   FRI 承诺方案演示（arkworks + BN254）                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let mut rng = StdRng::seed_from_u64(42);

    // ─── 1. 多项式 ─────────────────────────────────────────────────────
    // p(x) = 2x³ + 3x² + 4x + 5
    let poly = DensePolynomial::from_coefficients_slice(&[
        Fr::from(5),
        Fr::from(4),
        Fr::from(3),
        Fr::from(2),
    ]);
    println!("  多项式:    p(x) = 2x³ + 3x² + 4x + 5");
    println!("  度数:      degree = {}", poly.degree());
    println!();

    // ─── 2. 域 ─────────────────────────────────────────────────────────
    let n = 8usize;
    let domain = Radix2EvaluationDomain::<Fr>::new(n).unwrap();
    println!("  求值域大小: n = {}", n);
    println!(
        "  本原根 ω:   {:?}",
        Fr::get_root_of_unity(n as u64).unwrap()
    );
    println!();

    // ─── 3. FRI 证明 ──────────────────────────────────────────────────
    println!("  ─── FRI 协议执行 ───");
    let (proof, _trees) = fri_prove(&poly, &domain, &mut rng);
    println!();
    println!("  折叠轮数: {}", proof.rounds.len());
    for (i, &(ref r, beta)) in proof.rounds.iter().enumerate() {
        println!("    层 {i}: Merkle 根 = {:?}..., β = {beta:?}", &r[..4]);
    }
    println!("  最终常数: {:?}", proof.final_constant);
    println!();

    // ─── 4. 验证 ──────────────────────────────────────────────────────
    println!("  ─── FRI 验证 ───");
    if fri_verify(&proof, &domain) {
        println!("  ✅ FRI 证明验证通过!");
    } else {
        println!("  ❌ FRI 证明验证失败!");
    }
    println!();

    // ─── 5. 原理说明 ──────────────────────────────────────────────────
    println!("  ─── FRI 原理 ───");
    println!();
    println!("  [承诺]  在 8 个域点上求值 p(x) → Merkle 树根 = 承诺");
    println!();
    println!("  [折叠]  每轮将多项式度数折半:");
    println!("    p₀(x) = a₀ + a₁x + a₂x² + a₃x³");
    println!("         = (a₀ + a₂x²) + x·(a₁ + a₃x²)");
    println!("         = p_e(x²) + x·p_o(x²)");
    println!("    p₁(y) = p_e(y) + β·p_o(y)  (度数减半, y=x²)");
    println!();
    println!("  此例:");
    println!("    p₀(x) = 5 + 4x + 3x² + 2x³");
    println!("    p_e(y) = 5 + 3y ,  p_o(y) = 4 + 2y");
    println!("    p₁(y) = (5+3y) + β·(4+2y)");
    println!("          = (5+4β) + (3+2β)·y  (度数=1)");
    println!("    再折叠一次 → p₂(z) 为常数 (度数=0)");
    println!();
    println!("  [验证]  在查询点 i=0 检查每层的 Merkle 证明 + 折叠一致性:");
    println!("    p_{{r+1}}(ω²ʳⁱ) = (p_r(g) + p_r(-g))/2");
    println!("                    + β·(p_r(g) − p_r(-g))/(2·g)");
    println!("    其中 g = ω^(2ʳ·i)");
    println!();
    println!("  FRI 安全性: 验证者在每层检查同一个查询列, 确保折叠正确。");
    println!("  多轮查询 + 随机折叠挑战 β 使得作弊者无法伪造证明。");
}
