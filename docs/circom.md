# Circom

## Poseidon
Poseidon 是一种专门为 零知识证明 / 算术电路 设计的 哈希函数（hash）。在 Circom、Semaphore、Tornado、iden3 等生态里，它几乎是「默认哈希」。

内部主要是 加法和乘法，正好对应 R1CS 里的 + 和 ×

普通hash是为cpu设计的，主要是位运算。

##


Groth16 是一种极致优化证明大小和验证速度的 zk-SNARK 技术，通过配对密码学实现了“最小证明 + 最快验证”。
Groth16 把电路验证问题转化为“多项式是否能被整除”的问题，然后利用椭圆曲线配对密码学，把这个证明极致压缩成 3 个很小的点，从而实现证明体积小 + 验证速度极快。
系统需要在电路固定后进行一次可信设置（Trusted Setup）：

生成一些特殊的“公共参数”（CRS）。
这个过程像生成一把“特殊的锁”，只有知道正确答案的人才能做出匹配的“钥匙”。
一旦设置完成，后续所有证明都可以使用这把锁。
Trusted Setup 过程中，系统会生成一些秘密随机数（称为 toxic waste 或 trapdoor）。

如果这些秘密数字被任何人知道，就会发生灾难性后果：
这个人可以为任意虚假语句生成有效证明（即伪造证明）。
任何人（包括攻击者）都无法区分这个证明是真是假。
这相当于“掌握了后门钥匙”，可以随意伪造零知识证明。
这是 Groth16 最大的安全隐患。


Circom 使用它主要是因为在“链上验证成本”这个维度上，Groth16 目前仍然是性价比最高的选择，尤其适合需要频繁在区块链上验证证明的场景。

Circom 也可以使用plonk

## circom to solidity

在 Circom 生态中，你根本不需要手工去写 Solidity 验证代码。社区提供了一套完全成熟的、一键式命令工具，可以直接基于你编译出来的 Circom 电路约束（.zkey / 约束工件），全自动吐出一个生产环境可用的 Solidity 智能合约（通常命名为 verifier.sol）。

snarkjs zkey export solidityverifier your_circuit_final.zkey verifier.sol

