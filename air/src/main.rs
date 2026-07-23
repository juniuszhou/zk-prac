//! Plonky3 AIR 风格演示 — 以 a + b = c 加法电路为例
//!
//! 展示 Plonky3 中 AIR 的核心模式: BaseAir + Air trait + eval + AirBuilder

#![allow(dead_code)]
use std::{fmt, vec};

// ============================================================================
// 1. 域 (field)
// ============================================================================
type F = Fr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fr(u64);
const MOD: u64 = 101;

impl Fr {
    const fn new(x: u64) -> Self {
        Self(x % MOD)
    }
    fn add(self, rhs: Self) -> Self {
        Self((self.0 + rhs.0) % MOD)
    }
    fn sub(self, rhs: Self) -> Self {
        Self((self.0 + MOD - rhs.0) % MOD)
    }
    fn mul(self, rhs: Self) -> Self {
        Self((self.0 * rhs.0) % MOD)
    }
    const fn zero() -> Self {
        Self(0)
    }
    const fn one() -> Self {
        Self(1)
    }
}

impl fmt::Display for Fr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// 2. 模拟 Plonky3 的 AIR 核心 trait
// ============================================================================

/// Plonky3 的 `BaseAir` — 定义 trace 宽度、预置列、公开值数量
trait BaseAir {
    fn width(&self) -> usize;
    fn num_public_values(&self) -> usize {
        0
    }
}

/// Plonky3 的 `AirBuilder` — 提供 DSL: main, public_values, when_first_row 等
#[derive(Clone)]
struct AirBuilder<'a> {
    main_current: &'a [F],
    main_next: Option<&'a [F]>,
    public_values: &'a [F],
    row_index: usize,
    is_first: bool,
    is_last: bool,
    errors: Vec<String>,
}

impl<'a> AirBuilder<'a> {
    fn new(
        current: &'a [F],
        next: Option<&'a [F]>,
        pvs: &'a [F],
        row: usize,
        total: usize,
    ) -> Self {
        Self {
            main_current: current,
            main_next: next,
            public_values: pvs,
            row_index: row,
            is_first: row == 0,
            is_last: row == total - 1,
            errors: Vec::new(),
        }
    }

    fn main(&self) -> &[F] {
        self.main_current
    }

    fn next(&self) -> Option<&[F]> {
        self.main_next
    }

    fn public_values(&self) -> &[F] {
        self.public_values
    }

    fn assert_eq(&mut self, actual: F, expected: F) {
        if actual != expected {
            self.errors.push(format!(
                "row {}: expected {expected}, got {actual}",
                self.row_index
            ));
        }
    }

    fn assert_zero(&mut self, val: F) {
        if val != F::zero() {
            self.errors.push(format!(
                "row {}: constraint = {}, expected 0",
                self.row_index, val
            ));
        }
    }
}

/// Plonky3 的 `Air` trait — eval 方法定义所有约束
trait Air {
    fn eval(&self, builder: &mut AirBuilder);
}

// ============================================================================
// 3. 加法 AIR — 完全模仿 Plonky3 的写法
// ============================================================================

struct AdditionAir;

impl BaseAir for AdditionAir {
    fn width(&self) -> usize {
        3 // columns: a, b, c
    }
    fn num_public_values(&self) -> usize {
        1 // c 是公开的
    }
}

impl Air for AdditionAir {
    fn eval(&self, builder: &mut AirBuilder) {
        let main = builder.main();
        let a = main[0];
        let b = main[1];
        let c = main[2];

        println!(
            "  ── eval row {}: a = {}, b = {}, c = {}",
            builder.row_index, a, b, c,
        );

        // ── 门约束: a + b - c = 0  (每行都成立) ──
        builder.assert_zero(a.add(b).sub(c));

        // ── 最后一行的 c 必须等于公开值 ──
        if builder.is_last {
            let pv = builder.public_values()[0];
            builder.assert_eq(c, pv);
        }
    }
}

// ============================================================================
// 4. Trace 生成
// ============================================================================

#[derive(Clone)]
struct Trace {
    data: Vec<Vec<F>>, // row-major: 每行是一个 Vec<F>
    num_cols: usize,
}

impl Trace {
    fn new(num_rows: usize, num_cols: usize) -> Self {
        let data = vec![vec![F::zero(); num_cols]; num_rows];
        Self { data, num_cols }
    }

    fn set(&mut self, row: usize, col: usize, val: F) {
        self.data[row][col] = val;
    }

    fn num_rows(&self) -> usize {
        self.data.len()
    }

    fn push_row(&mut self, row: Vec<F>) {
        assert_eq!(row.len(), self.num_cols);
        self.data.push(row);
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ┌─────┬──────┬──────┬──────┐")?;
        writeln!(f, "  │ Row │ a    │ b    │ c    │")?;
        writeln!(f, "  ├─────┼──────┼──────┼──────┤")?;
        for (i, row) in self.data.iter().enumerate() {
            writeln!(
                f,
                "  │ {:3} │ {:4} │ {:4} │ {:4} │",
                i, row[0], row[1], row[2],
            )?;
        }
        writeln!(f, "  └─────┴──────┴──────┴──────┘")
    }
}

// ============================================================================
// 5. 验证流程 (模拟 Plonky3 prover/verifier 调用 `air.eval`)
// ============================================================================

fn verify(air: &impl Air, trace: &Trace, public_values: &[F]) -> Result<(), Vec<String>> {
    let mut all_errors = Vec::new();

    for i in 0..trace.num_rows() {
        println!("  ── 验证第 {} 行 ──", i);
        println!(" public_values: {:?}", public_values[i]);

        let public_values = &public_values[i..i + 1];

        let current = trace.data[i].as_slice();
        let next = if i + 1 < trace.num_rows() {
            Some(trace.data[i + 1].as_slice())
        } else {
            None
        };
        let mut builder = AirBuilder::new(current, next, &public_values, i, trace.num_rows());
        air.eval(&mut builder);
        all_errors.extend(builder.errors);
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}

// ============================================================================
// 6. main
// ============================================================================

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   Plonky3 AIR 风格演示: a + b = c                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Plonky3 的 AIR 由两个核心 trait 定义:\n");
    println!("  ┌─ BaseAir<F>");
    println!("  │   宽度, 预置列, 公开值数量, 周期列");
    println!("  │");
    println!("  └─ Air<AB: AirBuilder>");
    println!("       fn eval(&self, builder: &mut AB)");
    println!("       ┌─ builder.main()          ← 当前行数据");
    println!("       │  builder.next()          ← 下一行数据");
    println!("       │  builder.public_values() ← 公开输入");
    println!("       │  builder.when_first_row()");
    println!("       │  builder.when_transition()");
    println!("       │  builder.when_last_row()");
    println!("       │  builder.assert_eq(x, y)");
    println!("       └  builder.assert_zero(x)");
    println!();

    // ─── 构建加法 Trace ──────────────────────────────────────────────
    // 计算: 3 + 5 = 8
    // 单行: (3, 5, 8), 同时满足门约束和最后一行的公开值约束
    let air = AdditionAir;
    let public = vec![F::new(8), F::new(2)]; // 公开值: c = 8, a = 3, b = 5

    let mut trace = Trace::new(1, 3);
    trace.set(0, 0, F::new(3)); // a
    trace.set(0, 1, F::new(5)); // b
    trace.set(0, 2, F::new(8)); // c

    let row = vec![F::new(1), F::new(1), F::new(2)];
    trace.push_row(row);

    println!("  Trace: 3 + 5 = 8");
    println!("{trace}");

    match verify(&air, &trace, &public) {
        Ok(()) => println!("  ✅ Prover: AIR 约束全部满足 → 证明有效\n"),
        Err(e) => e.iter().for_each(|e| println!("  ❌ {e}")),
    }

    // ─── 对比: Plonky3 源码中的 Fibonacci AIR ──────────────────────────
    println!("  ─── Plonky3 Fibonacci AIR (来自 batch-stark/tests/simple.rs) ───");
    println!("  ┌─────────────────────────────────────────────────────────┐");
    println!("  │ impl<AB: AirBuilder> Air<AB> for FibonacciAir {{       │");
    println!("  │   fn eval(&self, builder: &mut AB) {{                  │");
    println!("  │     let main = builder.main();                         │");
    println!("  │     let pis = builder.public_values();                 │");
    println!("  │     let local: &FibRow = main.current_slice();         │");
    println!("  │     let next: &FibRow  = main.next_slice();            │");
    println!("  │                                                        │");
    println!("  │     // 边界约束: 第 0 行 left=a0, right=b0             │");
    println!("  │     builder.when_first_row()                           │");
    println!("  │       .assert_eq(local.left, pis[0]);                  │");
    println!("  │       .assert_eq(local.right, pis[1]);                 │");
    println!("  │                                                        │");
    println!("  │     // 转移约束: fib[i+1] = (fib[i].right, fib[i].sum) │");
    println!("  │     builder.when_transition()                          │");
    println!("  │       .assert_eq(local.right, next.left);              │");
    println!("  │       .assert_eq(local.left + local.right, next.right);│");
    println!("  │                                                        │");
    println!("  │     // 最后一行的 right 必须等于公开值 x               │");
    println!("  │     builder.when_last_row().assert_eq(local.right,x);  │");
    println!("  │   }}                                                   │");
    println!("  │ }}                                                     │");
    println!("  └─────────────────────────────────────────────────────────┘");
    println!();
    println!("  AIR 本质: eval() 里通过 builder 断言约束, builder 的实现");
    println!("  (ProverConstraintFolder / VerifierConstraintFolder) 决定");
    println!("  约束如何被编译成 STARK 证明或验证。");
}
