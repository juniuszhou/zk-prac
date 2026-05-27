# Zero-Knowledge Expert Roadmap

This roadmap is designed for a developer with Solidity experience who wants to become strong in zero knowledge, then move toward zkML with Circom, Halo2, and EZKL.

## Phase 1: ZK Foundations

Goal: understand what a zero-knowledge proof proves before relying on tools.

Study:

- Finite fields: ZK circuits compute over numbers modulo a large prime.
- Statements and witnesses: the verifier checks a public statement; the prover knows private witness values.
- Constraints: programs become algebraic equations the witness must satisfy.
- Completeness: honest proofs for true statements verify.
- Soundness: false statements cannot be proven except with negligible probability.
- Zero knowledge: the proof leaks nothing beyond statement validity.
- Polynomial viewpoint: modern SNARKs reduce constraints to polynomial identities.

Practice:

- Write an `a + b = c` circuit.
- Generate valid and invalid witnesses.
- Explain which values are public and private.
- Count the constraints by hand.

## Phase 2: Circom And R1CS

Goal: build small circuits and understand their cost.

Study:

- `signal input`, public input lists on `main`, and intermediate signals.
- `<==` witness assignment plus constraint generation.
- `===` explicit equality constraints.
- R1CS form: each constraint is roughly `(A * B) = C`.
- Why multiplication is expensive and linear arithmetic is cheap.
- Underconstrained circuit bugs.

Practice:

- Range checks.
- Comparators.
- Hash preimage checks.
- Merkle tree membership.
- Nullifier generation.

## Phase 3: Proving Systems

Goal: choose the right proof system for the application.

Study:

- Groth16: tiny proofs and cheap verification, but circuit-specific trusted setup.
- PLONK and Halo2: flexible arithmetization, custom gates, lookup tables.
- STARKs: transparent setup and FRI-based commitments, usually larger proofs.
- zkVMs: prove program execution instead of writing every constraint manually.
- Recursion and folding: aggregate or incrementally prove many computations.

Practice:

- Export a Solidity verifier for a Groth16 Circom circuit.
- Compare proof size, proving time, verifier gas, and setup requirements.
- Read a small Halo2 circuit and identify gates, advice columns, fixed columns, and selectors.

## Phase 4: Real Applications

Goal: build end-to-end systems where proofs are verified on-chain or by a backend.

Build:

- Private allowlist with Merkle membership.
- Anonymous voting with nullifiers.
- Private balance or solvency proof.
- zk login or identity proof.
- Simple zkML inference proof with EZKL.

For each project, document:

- Circuit logic.
- Public inputs.
- Private witness.
- Constraint count estimate.
- Test cases.
- Solidity verifier usage, when applicable.

## Phase 5: Expert Topics

Goal: review and design production-grade ZK systems.

Study:

- Polynomial commitment schemes: KZG, IPA, FRI.
- Lookup arguments and why they help range checks, bit operations, and ML quantization.
- Constraint optimization and witness generation performance.
- Recursive SNARKs.
- Folding schemes such as Nova-style systems.
- Security review patterns for underconstrained circuits.
- Trusted setup operational risks.

Practice:

- Audit a simple circuit for missing constraints.
- Optimize a circuit and measure constraint reduction.
- Build an ONNX to EZKL flow for a small MLP model.
- Generate a Solidity verifier and write an end-to-end test.




Month 1: 基础 + zkVM 入门（打牢根基）
Week 1-2: Rust + RISC-V 基础

Rust 进阶： ownership 深挖、unsafe Rust、性能优化（rayon、SIMD）、no_std 环境。
RISC-V ISA：学习 RV32IM（基础整数 + 乘法），理解寄存器、指令周期、内存模型。
资源：RISC-V Reader（免费 PDF）、edX “Building a RISC-V CPU Core”。

实践：用 Rust 写简单 RISC-V 模拟器（加法器、简单 CPU cycle）。

Week 3-4: zkVM 入门 + RISC Zero 实战

核心概念：Guest Program、Host、Execution Trace、Arithmetization、STARK 证明。
学习路径：
安装 rzup + cargo-risczero，跑 Hello World。
完成官方 Tutorial（dev.risczero.com）：Sudoku、JSON、简单算法。
深入理解 zkVM 架构：如何把 RISC-V 指令转为约束（constraints）。

资源：
RISC Zero 官方文档 + Study Club 视频。
GitHub: risc0/risc0 examples。

实践项目：用 Rust 实现一个可证明的 Merkle Tree 或排序算法，并生成证明。

目标输出：能独立写 Guest 程序，理解证明生成流程。
Month 2: 深入证明系统 + SP1 对比（核心技术）
Week 5-6: 证明系统理论

重点学习：STARK（FRI、AIR）、SNARK、Plonkish、Lookup Argument。
RISC Zero 的 Zirgen DSL 和 STARK 实现。
资源：
RISC Zero Proof System 论文（Scalable Transparent Arguments of RISC-V）。
Vitalik 的 STARK 系列文章、zkSecurity 的 STARK Book。

实践：阅读 RISC Zero 源码中 Prover/Verifier 部分。

Week 7-8: SP1 + 性能优化

学习 SP1（Succinct Labs）：Hypercube 架构、Precompiles、GPU 加速。
对比 RISC Zero vs SP1：Precompile 设计、递归证明、实时证明（real-time proving）。
实践：
用 SP1 实现 Ethereum 轻客户端证明或简单 zkML 推理。
优化一个 Guest 程序的证明时间（添加 precompiles）。

资源：docs.succinct.xyz + SP1 GitHub examples。

目标输出：能 debug zkVM 执行 trace，理解证明瓶颈在哪里。
Month 3: 高级主题 + 开源贡献（向核心靠拢）
Week 9-10: 高级特性

Precompiles 开发（加速哈希、ECDSA、大整数）。
递归证明（Proof Composition）、Formal Verification（Lean）。
硬件加速（GPU/FPGA 在 Prover 中的作用）。
zkVM 应用集成：OP Succinct、Rollup 证明、跨链。

Week 11-12: 项目实战 + 贡献

选择一个项目深入：
RISC Zero：修复 docs、优化 examples、贡献小 feature。
SP1：Precompile 优化、bug fix。

查找 Good First Issues（GitHub labels: good first issue）。
实践大项目：实现一个完整应用（如可验证 JSON 查询引擎或区块链状态证明），部署到 Bonsai 或本地 Prover。
参与社区：Discord/Telegram、Study Club、GitHub Discussions。

目标输出：

至少 1-2 个有意义的 PR（即使是文档或测试）。
能独立分析 zkVM 性能瓶颈并提出优化方案。

推荐资源汇总（2026 年最新）

文档：dev.risczero.com（最完整）、docs.succinct.xyz。
代码：risc0/risc0、succinctlabs/sp1。
理论：RISC Zero 白皮书、STARK 论文。
社区：RISC Zero Study Club、Succinct Telegram。