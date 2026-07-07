# snark


## 
在现代零知识证明（ZKP）的模块化设计中，一个完整的证明系统（Proving System）通常由两部分解耦组合而成：

前端：代数化/约束方案（Arithmetization / Constraint Scheme） —— 负责把业务逻辑变成数学方程（比如：R1CS、Plonkish、AIR）。

后端：验证协议/信息论协议 + 承诺方案（Information-Theoretic IOP + Commitment Scheme） —— 负责用密码学手段证明这些数学方程确实成立。

案例一：同样的“约束方案”，换个“后端”就是全新的协议
Plonkish（约束方案） + KZG（承诺后端） = 原始 PLONK / Scroll / Axiom

特点： 证明体积小，以太坊上验证便宜，但需要 Universal Setup（通用可信设置）。

Plonkish（约束方案） + IPA（承诺后端） = Halo2 (Zcash)

特点： 彻底去除了可信设置（No Trust Setup），但证明生成和验证的计算开销有所不同。

Plonkish（约束方案） + FRI（承诺后端） = Plonky2 / Plonky3

特点： 变成了基于哈希函数的 STARK 式后端，生成证明的速度极快（因为避开了昂贵的椭圆曲线配对），但证明体积变大。

案例二：同样的“后端”，可以用来验证不同的“约束方案”
KZG 承诺（后端） 既可以用来支持 Plonkish 约束系统（如标准 PLONK），也可以用来支持 R1CS 约束系统（如 Groth16 的变体或 Marlin 协议）。


## snark 一般流程
snark 是一个框架，包含下面几个过程。每个过程可能使用不同的技术
计算  （原始程序， rust python solidity）
 ↓
约束系统 （R1CS PLONKish AIR） constraint
 ↓
多项式表示 （QAP  Gate Selector Ploynomia ） constraint mapping to parameters of polynomia
 ↓
承诺方案 （PCS KZG IPA FRI）ploynomia locked and computing 
 ↓
证明协议
 ↓
验证协议 pairing based R1CS， Plonk arithmetization 