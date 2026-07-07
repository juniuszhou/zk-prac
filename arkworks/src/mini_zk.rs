//! # Mini ZK System — 基于 arkworks 组件的逐步教学实现
//!
//! ## 依赖链
//! ```text
//! ark-ff (第1层) —— 有限域：所有 ZK 算术的基石
//!   ↓
//! ark-poly (第2层) —— 多项式：QAP 约束→多项式转换
//!   ↓
//! ark-poly-commit (第3层) —— KZG 多项式承诺：简洁求值证明
//!   ↓
//! ark-relations (第4层) —— R1CS 约束系统：编码计算
//!   ↓
//! ark-groth16 (第5层) —— Groth16 zk-SNARK：Setup / Prove / Verify
//! ```
//!
//! ## ZK 证明的阶段对应
//! - **Setup（可信设置）**: 第5层 Groth16 生成 pk/vk；第3层 KZG 生成 CRS
//! - **Prove（证明生成）**: 第4层构建 R1CS → 第5层生成 Groth16 证明
//! - **Verify（证明验证）**: 第5层验证 Groth16 证明；第3层验证 KZG 求值证明
//!
//! 每个数据结构/函数都标注了其含义和对应 ZK 证明步骤。

use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::{Field, One, UniformRand};
use ark_groth16::{Groth16, PreparedVerifyingKey};
use ark_poly::univariate::DensePolynomial;
use ark_poly::{
    DenseUVPolynomial, EvaluationDomain, Evaluations, Polynomial, Radix2EvaluationDomain,
};
use ark_poly_commit::kzg10::{Powers, Proof, VerifierKey, KZG10};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, LinearCombination,
    SynthesisError, Variable,
};
use ark_snark::SNARK;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::borrow::Cow;

// ============================================================================
// 类型别名 - 让代码更简洁
// ============================================================================

/// BLS12-381 标量域上的单变元稠密多项式类型
type UniPoly = DensePolynomial<Fr>;

/// KZG10 承诺方案，用 BLS12-381 配对曲线 + 稠密多项式
type KZG = KZG10<Bls12_381, UniPoly>;

// ============================================================================
// 第1层: 有限域 (ark-ff)
// ============================================================================
// 有限域是 ZK 的数学基石。BLS12-381 的标量域 Fr 是一个素数域，所有 ZK 计算
// 都在这个域中进行（模素数运算）。
//
// 对应 ZK 阶段: 所有阶段的底层基础设施

pub fn layer1_field_demo() {
    println!("\n━━━ 第1层: 有限域 (ark-ff) ━━━");
    println!("    ZK 基础: 所有算术在有限域 Fr 中完成\n");

    let mut rng = StdRng::seed_from_u64(42);

    // 创建域元素: Fr::from(u64) 将整数转换为域元素
    let a = Fr::from(10_u64);
    let b = Fr::from(3_u64);

    // 域运算: 加法、减法、乘法、除法
    let sum = a + b; // 13
    let diff = a - b; // 7
    let prod = a * b; // 30
    let quot = a / b; // 10/3 (域除法相当于乘以乘法逆元)

    println!("  a = {a:?}, b = {b:?}");
    println!("  a + b = {sum:?}  (加法)");
    println!("  a - b = {diff:?}  (减法)");
    println!("  a * b = {prod:?}  (乘法)");
    println!("  a / b = {quot:?}  (域除法 = a * b⁻¹)");

    // 域的封闭性: 任何运算结果仍在域中
    let inv = b.inverse().unwrap(); // b 的乘法逆元
    assert_eq!(b * inv, Fr::one()); // b * b⁻¹ = 1
    println!("  b⁻¹ * b = 1  验证通过 ✓");

    // 随机域元素
    let rand_elem = Fr::rand(&mut rng);
    println!("  随机域元素: {rand_elem:?}");
    println!("  ✓ 第1层通过\n");
}

// ============================================================================
// 第2层: 多项式 (ark-poly)
// ============================================================================
// 多项式在 ZK 中的关键作用: QAP(Quadratic Arithmetic Program)将 R1CS 约束
// 转换成多项式方程，使得验证者可以通过检查多项式在一个随机点的求值来验证
// 所有约束是否满足。
//
// 对应 ZK 阶段: 约束→多项式转换 (R1CS→QAP)

pub fn layer2_poly_demo() {
    println!("\n━━━ 第2层: 多项式 (ark-poly) ━━━");
    println!("    ZK 作用: R1CS→QAP 约束→多项式转换\n");

    // DensePolynomial: 用系数向量表示多项式 a₀ + a₁x + a₂x² + ...
    // 这里构造 x² + 2x + 3
    let poly = DensePolynomial::from_coefficients_slice(&[
        Fr::from(3_u64), // 常数项
        Fr::from(2_u64), // x项
        Fr::from(1_u64), // x²项
    ]);
    println!("  p(x) = x² + 2x + 3");

    // evaluate: 在给定点求多项式值
    let point = Fr::from(5_u64);
    let value = poly.evaluate(&point);
    //   p(5) = 25 + 10 + 3 = 38
    println!("  p(5) = {value:?}  (多项式求值)");
    assert_eq!(value, Fr::from(38_u64));

    // Radix2EvaluationDomain: 大小为 2ⁿ 的求值域，用于 FFT/插值
    // 这是 QAP 转换的基础: 在 N 个点上插值 N 个约束
    let domain_size = 4;
    let domain = Radix2EvaluationDomain::<Fr>::new(domain_size)
        .expect("域大小必须是 2 的幂且不超过 Fr 的 2-adic 阶");

    // Evaluations: 多项式在某组点上的求值表示
    let evals = domain
        .elements()
        .map(|x| poly.evaluate(&x))
        .collect::<Vec<_>>();
    let eval_form = Evaluations::from_vec_and_domain(evals.clone(), domain);

    // interpolate: 从求值形式恢复系数形式（逆 FFT）
    // 这是在 QAP 中构建多项式的方法: 约束值→多项式
    let recovered = eval_form.interpolate();
    println!("  插值恢复 | p(x) = {}", poly == recovered);

    // 验证: 插值得到的多项式在域点上与原多项式一致
    for (i, x) in domain.elements().enumerate() {
        assert_eq!(poly.evaluate(&x), recovered.evaluate(&x));
        println!("    p(ω^{i}) = {:?}  ✓", poly.evaluate(&x));
    }
    println!("  ✓ 第2层通过\n");
}

// ============================================================================
// 第3层: KZG 多项式承诺 (ark-poly-commit)
// ============================================================================
// 多项式承诺允许证明者:
//   1. Commit: 承诺一个多项式 p(x)，得到一个简洁的承诺值 C（群元素）
//   2. Open: 在任意点 z 打开承诺，给出 p(z) 的值和证明
//   3. Verify: 验证者检查证明，确信 p(z) 确实被 C 承诺
//
// KZG10 基于配对的方案:
// - 设置: 生成 (g, g^s, g^{s²}, ..., h, h^s) 等公共参考串（CRS）
// - 承诺: C = g^{p(s)} = ∏ (g^{s^i})^{a_i}
// - 证明: π = g^{q(s)}，其中 q(x) = (p(x)-p(z))/(x-z)
// - 验证: e(C - g^{p(z)}, h) = e(π, h^s - h^z)
//
// 对应 ZK 阶段: 简洁证明的底层原语，Groth16 等 SNARK 的核心组件

pub fn layer3_kzg_demo() {
    println!("\n━━━ 第3层: KZG 多项式承诺 (ark-poly-commit) ━━━");
    println!("    ZK 作用: 简洁承诺 + 求值证明\n");

    let mut rng = StdRng::seed_from_u64(42);

    // ── Step 1: Setup ──────────────────────────────────────────────
    // KZG10::setup(max_degree, produce_g2_powers, rng) -> UniversalParams
    // 生成 CRS = (g, g^s, g^{s²}, ..., h, h^s)
    // 安全要求: s 必须被丢弃（可信设置）
    let max_degree = 8;
    let pp = KZG::setup(max_degree, false, &mut rng).expect("KZG setup 失败");
    println!("  [Setup] 生成 CRS (max_degree={max_degree})");

    // ── Step 2: Trim ────────────────────────────────────────────────
    // 从 UniversalParams 中提取当前度数需要的 Powers 和 VerifierKey
    // Powers: 给证明者使用的群元素
    // VerifierKey: 给验证者使用的配对验证Key
    let supported_degree = 4;
    let powers_of_g = pp.powers_of_g[..=supported_degree].to_vec();
    let powers_of_gamma_g: Vec<_> = (0..=supported_degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();
    let powers = Powers {
        powers_of_g: Cow::Owned(powers_of_g),
        powers_of_gamma_g: Cow::Owned(powers_of_gamma_g),
    };
    let vk = VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };
    println!("  [Trim] 提取 degree≤{supported_degree} 的密钥");

    // ── Step 3: 创建多项式 ─────────────────────────────────────────
    // p(x) = 2x² + 3x + 7
    let poly = DensePolynomial::from_coefficients_slice(&[
        Fr::from(7_u64),
        Fr::from(3_u64),
        Fr::from(2_u64),
    ]);
    println!("  p(x) = 2x² + 3x + 7");

    // ── Step 4: Commit ──────────────────────────────────────────────
    // 生成承诺 C = g^{p(s)}，这是多项式 p 的绑定承诺
    // hiding_bound=None 表示不隐藏多项式
    let (comm, _rand) = KZG::commit(
        &powers,
        &poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .expect("KZG commit 失败");
    println!("  [Commit] C = g^p(s) = {:?}", comm.0);

    // ── Step 5: 求值 ────────────────────────────────────────────────
    // 计算 v = p(z)，在点 z=5 处求值
    let z = Fr::from(5_u64);
    let v = poly.evaluate(&z);
    // p(5) = 2*25 + 15 + 7 = 72
    println!("  [Eval] p({z:?}) = {v:?}");

    // ── Step 6: 创建求值证明 ──────────────────────────────────────
    // compute_witness_polynomial: 计算 q(x) = (p(x)-p(z))/(x-z)
    // 这是 KZG 证明的核心: 证明者构造商多项式
    let (witness_poly, _) =
        KZG::compute_witness_polynomial(&poly, z, &_rand).expect("计算 witness 多项式失败");
    println!("  q(x) = (p(x)-p(z))/(x-z)     (商多项式)");

    // 对 witness 多项式做承诺得到证明 π = g^{q(s)}
    let (comm_w, _) = KZG::commit(
        &powers,
        &witness_poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .expect("KZG commit witness 失败");
    let proof = Proof {
        w: comm_w.0,
        random_v: None,
    };
    println!("  [Prove] π = g^q(s) = {:?}", proof.w);

    // ── Step 7: Verify ──────────────────────────────────────────────
    // 验证: e(C - g^v, h) == e(π, h^s - h^z)
    // 通过配对检验商多项式的正确性
    let ok = KZG::check(&vk, &comm, z, v, &proof).expect("KZG verify 失败");
    println!("  [Verify] e(C - g^v, h) == e(π, h^β - h^z) → {ok}");

    assert!(ok, "KZG 证明验证应该通过");
    println!("  ✓ 第3层通过\n");
}

// ============================================================================
// 第4层: R1CS 约束系统 (ark-relations)
// ============================================================================
// R1CS (Rank-1 Constraint System) 是 zk-SNARK 的中介表示:
//   ⟨A, w⟩ · ⟨B, w⟩ = ⟨C, w⟩
// 其中 w = (1, public_inputs, private_witnesses) 是完整见证向量
//
// 对应 ZK 阶段: 将计算问题编码为约束系统

pub fn layer4_r1cs_demo() {
    println!("\n━━━ 第4层: R1CS 约束系统 (ark-relations) ━━━");
    println!("    ZK 作用: 将计算编码为 ⟨A,w⟩·⟨B,w⟩ = ⟨C,w⟩\n");

    // ── 示例: x³ + x + 5 = out ─────────────────────────────────────
    // 见证变量布局:
    // 索引 0: ONE（隐式常数 1）
    // 索引 1: out      (public instance)
    // 索引 2: x        (private witness 0)
    // 索引 3: sym_1    (private witness 1, = x*x)
    // 索引 4: y        (private witness 2, = sym_1*x)
    // 索引 5: sym_2    (private witness 3, = y+x)
    //
    // 约束:
    // 1) x * x = sym_1
    // 2) sym_1 * x = y
    // 3) (y + x) * 1 = sym_2
    // 4) (sym_2 + 5) * 1 = out

    let cs = ConstraintSystem::<Fr>::new_ref();

    // 分配变量
    // new_input_variable: 公开输入（instance）
    let out = cs
        .new_input_variable(|| Ok(Fr::from(35_u64)))
        .expect("分配 out");
    // new_witness_variable: 私有见证（witness）
    let x = cs
        .new_witness_variable(|| Ok(Fr::from(3_u64)))
        .expect("分配 x");
    let sym_1 = cs
        .new_witness_variable(|| Ok(Fr::from(9_u64)))
        .expect("分配 sym_1");
    let y = cs
        .new_witness_variable(|| Ok(Fr::from(27_u64)))
        .expect("分配 y");
    let sym_2 = cs
        .new_witness_variable(|| Ok(Fr::from(30_u64)))
        .expect("分配 sym_2");

    // 约束 1: x * x = sym_1
    // ⟨A,w⟩ = x, ⟨B,w⟩ = x, ⟨C,w⟩ = sym_1
    cs.enforce_constraint(
        LinearCombination::from(x),
        LinearCombination::from(x),
        LinearCombination::from(sym_1),
    )
    .expect("约束 1 失败");

    // 约束 2: sym_1 * x = y
    cs.enforce_constraint(
        LinearCombination::from(sym_1),
        LinearCombination::from(x),
        LinearCombination::from(y),
    )
    .expect("约束 2 失败");

    // 约束 3: (y + x) * 1 = sym_2
    cs.enforce_constraint(
        LinearCombination::from(y) + x,
        LinearCombination::from(Variable::One),
        LinearCombination::from(sym_2),
    )
    .expect("约束 3 失败");

    // 约束 4: (sym_2 + 5) * 1 = out
    cs.enforce_constraint(
        LinearCombination::from(sym_2) + (Fr::from(5_u64), Variable::One),
        LinearCombination::from(Variable::One),
        LinearCombination::from(out),
    )
    .expect("约束 4 失败");

    cs.finalize();

    // is_satisfied: 检查所有约束是否满足（见证是否有效）
    let satisfied = cs.is_satisfied().expect("检查失败");
    println!("  R1CS 约束数: {}", cs.num_constraints());
    println!(
        "  实例变量: {}\n  见证变量: {}",
        cs.num_instance_variables(),
        cs.num_witness_variables()
    );
    println!("  is_satisfied(): {satisfied}");
    assert!(satisfied, "R1CS 约束应该全部满足");

    // matrices: 导出 A, B, C 稀疏矩阵，可用于 QAP 转换
    let matrices = cs.to_matrices().expect("导出矩阵失败");
    let (m, n) = (
        matrices.num_constraints,
        matrices.num_instance_variables + matrices.num_witness_variables,
    );
    println!("  矩阵维度: {m} 约束 × {n} 列\n  ✓ 第4层通过\n");
}

// ============================================================================
// 第5层: Groth16 zk-SNARK (ark-groth16)
// ============================================================================
// Groth16 是一个高效的 zk-SNARK 协议:
// - 证明大小: 3 个群元素（常量）
// - 验证: 2 个配对（预计算后）
// - 安全: 在随机预言机模型下证明知识可靠
//
// 对应 ZK 阶段:
//   1. Setup: 为电路生成证明密钥 pk 和验证密钥 vk
//   2. Prove: 生成关于 (x + y = z) 的零知识证明
//   3. Verify: 验证者只用公开输入 z 和证明即可验证

/// 加法电路: 证明知道 a, b 使得 a + b = c
/// - 公开输入: c
/// - 私有见证: a, b
#[derive(Clone)]
struct AddCircuit {
    a: Option<Fr>,
    b: Option<Fr>,
    c: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for AddCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // 私有见证: a, b（证明者知道但不公开）
        let a = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;
        // 公开输入: c（验证者也知道）
        let c = cs.new_input_variable(|| self.c.ok_or(SynthesisError::AssignmentMissing))?;

        // 约束: (a + b) * 1 = c
        cs.enforce_constraint(
            LinearCombination::from(a) + b,
            LinearCombination::from(Variable::One),
            LinearCombination::from(c),
        )?;
        Ok(())
    }
}

pub fn layer5_groth16_demo() {
    println!("\n━━━ 第5层: Groth16 zk-SNARK (ark-groth16) ━━━");
    println!("    ZK 作用: 完整的零知识证明系统 (Setup→Prove→Verify)\n");

    let mut rng = StdRng::seed_from_u64(42);

    // ── 定义电路实例 ──────────────────────────────────────────────
    // 证明者知道 a=3, b=5，声称 a+b=8
    // 公开输出 c=8，私有见证 a=3, b=5
    let circuit = AddCircuit {
        a: Some(Fr::from(3_u64)),
        b: Some(Fr::from(5_u64)),
        c: Some(Fr::from(8_u64)),
    };
    println!("  电路: a + b = c");
    println!("  私有见证: a=3, b=5");
    println!("  公开输入: c=8");

    // ── Step 1: Setup ──────────────────────────────────────────────
    // circuit_specific_setup: 为指定电路生成证明/验证密钥
    // 这是一个可信设置过程（需要安全地丢弃随机数）
    let (pk, vk) =
        <Groth16<Bls12_381> as SNARK<Fr>>::circuit_specific_setup(circuit.clone(), &mut rng)
            .expect("Groth16 setup 失败");
    println!("  [Setup] 生成 pk (证明密钥) + vk (验证密钥)");

    // ── Step 2: Prove ──────────────────────────────────────────────
    // 用 pk 和电路实例生成证明
    let proof = <Groth16<Bls12_381> as SNARK<_>>::prove(&pk, circuit, &mut rng)
        .expect("Groth16 prove 失败");
    println!("  [Prove] 生成证明 ({:?})", proof);

    // ── Step 3: Verify ─────────────────────────────────────────────
    // 验证者预处理 vk 以提高验证速度
    let pvk = PreparedVerifyingKey::<Bls12_381>::from(vk);
    let public_input = vec![Fr::from(8_u64)];

    let verified =
        <Groth16<Bls12_381> as SNARK<_>>::verify_with_processed_vk(&pvk, &public_input, &proof)
            .expect("Groth16 verify 失败");
    println!("  [Verify] 公开输入 c=8 → 验证结果: {verified}");
    assert!(verified, "Groth16 证明验证应该通过");
    println!("  ✓ 第5层通过\n");
}

// ============================================================================
// 主函数: 运行所有层
// ============================================================================
pub fn run_mini_zk_demo() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          Mini ZK System — 逐步教学演示                      ║");
    println!("║  依赖链: ark-ff → ark-poly → ark-poly-commit →             ║");
    println!("║          ark-relations → ark-groth16                       ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    layer1_field_demo();
    layer2_poly_demo();
    layer3_kzg_demo();
    layer4_r1cs_demo();
    layer5_groth16_demo();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          全部 5 层验证通过! ✓✓✓✓✓                         ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}
