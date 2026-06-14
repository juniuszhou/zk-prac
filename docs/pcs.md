# Polynomial Commitment Scheme

Commit（承诺）：Prover 用某种密码学哈希或椭圆曲线算法，把这个巨大的多项式 $P(x)$ 压缩成一个极小的、固定大小的哈希值或曲线点（通常只有 32 到 64 字节）。这个值就像是多项式的“数字指纹”，叫作 Commitment（承诺）。Prover 把这个指纹先发给 Verifier。

Open / Evaluation（挑战与证明）：Verifier 随机选择一个点 $\zeta$（读作 Zeta），挑战 Prover：“请告诉我，你的多项式在 $x = \zeta$ 处的值 $y$ 是多少？并证明它确实契合你刚才给我的指纹。” Prover 计算出 $y = P(\zeta)$，并生成一个很小的 Evaluation Proof（求值证明） 发过去。



## KZG
KZG (Kate-Zaverucha-Goldberg)
密码学底座：配对友好型椭圆曲线（Pairing-friendly EC，如 BN254, BLS12-381）。

## IPA
IPA (Inner Product Argument / 内积论证)
密码学底座：普通的椭圆曲线（不需要支持配对，如 Secp256k1 或 Curve25519）。

工作原理：通过一种类似折纸（Folding）的递归减半策略，将一个大向量的内积一步步压缩。

## FRI
FRI (Fast Reed-Solomon Interactive Oracle Proof of Proximity)
密码学底座：纯哈希函数（Hash Functions，如 SHA-256、Keccak、Poseidon）。

工作原理：它严格来说是一个“邻近性测试”。它不把多项式放在椭圆曲线上，而是把多项式在大量点上的求值结果，用 Merkle Tree（默克尔树） 一层层压紧，通过纠错码（Reed-Solomon）理论来证明这堆数据确实来自一个“低次多项式”。

