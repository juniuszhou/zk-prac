# zkvm

使用完全不同的方法来证明，它的作用是提供cpu执行的trace，来证明计算确实在VM执行过。
执行的VM多使用RISC-V来做。

https://jolt.a16zcrypto.com/


sp1
## risc zero 

1. 代数化与约束方案（如何把计算变成方程）
RISC Zero 是一个 zkVM（零知识虚拟机），它模拟的是标准的 RISC-V (RV32IM) 指令集架构。

约束表达： 它使用的是类似 STARK 体系常用的 AIR（Algebraic Intermediate Representation，代数中间表示），并结合了类似于 Plonkish 的列布局思路。

基本原理： 当你的 Rust 代码编译成 RISC-V 机器码并在虚拟机中执行时，RISC Zero 会记录一个极其庞大的执行轨迹表（Execution Trace）。这个表格的每一行代表一个 CPU 时钟周期，列则代表寄存器状态、内存读写和控制信号。
程序会在纵向上为这些列施加多项式约束（比如证明：当前行的程序计数器 PC + 4 = 下一行的 PC）。此外，它还引入了 PLOOKUP（查找表） 机制来高效处理复杂的内存读写一致性检查。

2. 承诺技术（Commitment Scheme）
既然核心是 STARK，RISC Zero 的多项式承诺自然也遵循 STARK 体系：

核心承诺方案：基于哈希函数的默克尔树（Merkle Tree）承诺。

它并不依赖诸如 KZG 那样昂贵的椭圆曲线双线性配对，而是直接把执行轨迹多项式在特定域上的求值结果拼成一个巨大的 Merkle Tree，然后将根哈希（Root Hash）作为承诺。

底层哈希函数：

在进行基础的虚拟机执行证明时，它主要使用传统的 SHA-256。

在进行递归证明（Recursion）和准备链上验证时，为了对电路更友好，它会切换到 ZK 友好的 Poseidon 哈希函数。

特点： 这种基于哈希的承诺方案使得 RISC Zero 不需要任何可信设置（No Trusted Setup / Transparent），并且具备抗量子计算机攻击的特性。

3. 验证与证明技术（Proving & Verification）
RISC Zero 的证明生成和验证过程经历了一个三阶段的“接力赛”：

阶段一：基础 STARK 证明（FRI 协议）
在证明者（Prover）把执行轨迹转换成多项式并做完 Merkle 承诺后，它需要向验证者证明“这些多项式的度（Degree）是非常低的（即确实符合计算规则）”。

核心验证技术： 使用 DEEP-ALI 协议 和 批处理 FRI 协议（Fast Reed-Solomon Interactive Oracle Proof of Proximity）。

结果： 这一步会生成一个原生的 STARK 证明（Seal）。它的生成速度极快（支持 GPU 深度加速），但缺点是证明体积非常大（通常有几百 KB 到上 MB），如果直接发到以太坊上验证，Gas 费会贵到无法承受。

阶段二：递归聚合（STARK Recursion）
为了解决证明体积大、以及大型程序执行时间长的问题，RISC Zero 把程序切成多个片段（Segments）并行生成多个 STARK 证明。

核心验证技术： 编写一个专门用来验证 STARK 的 STARK 电路。通过递归（Recursion），把多个片段的 STARK 证明像搭积木一样，不断两两合并，最终压缩成一个单一的、简洁的 Succinct STARK 证明。

阶段三：外层 SNARK 包裹（Groth16 Wrap）
为了让以太坊等区块链能够廉价地验证这个证明，RISC Zero 做了最后一步“格式转换”：

核心验证技术： 最终的 Succinct STARK 证明会被送入一个 Groth16 协议（基于 BN254 椭圆曲线） 的 R1CS 电路中。

结果： 经过 Groth16 的重新包裹，原本几百 KB 的 STARK 证明被瞬间压缩成了一个只有 几百个字节 的 Groth16 SNARK 证明。在链上验证它只需要消耗大约 20 多万 Gas。

注意： 因为最后一步用了 Groth16，所以 RISC Zero 针对这个“包裹电路”进行过一次通用的可信设置（Trusted Setup）仪式。