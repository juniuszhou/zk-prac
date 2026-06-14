# zk

## Trust setup

Powers of Tau（Tau 的幂次） 是可信设置（Trusted Setup）的核心产物。

简单来说，Powers of Tau 是用来对电路多项式进行加密求值的“秘密参数集合”。

在基于 R1CS（约束系统）的零知识证明（如 Groth16）中，Powers of Tau（Tau 的幂次） 是可信设置（Trusted Setup）的核心产物。简单来说，Powers of Tau 是用来对电路多项式进行加密求值的“秘密参数集合”。为了让你彻底理解它的作用，我们需要从 ZK 的底层逻辑来看它解决了什么问题。

1. 核心矛盾：如何证明我知道多项式，却不泄露多项式？在 R1CS 转换为 QAP（二次算术程序）后，证明 ZK 问题的核心变成了：证明者（Prover）需要向验证者（Verifier）证明自己知道一个多项式 $P(x)$，使得它能被目标多项式 $T(x)$ 整除。根据密码学中的 Schwartz-Zippel 引理：两个不同的多项式，在随机一点 $s$ 上求值，结果相等的概率极低。因此，验证者只需要挑一个随机点 $\tau$（读作 Tau），看 $P(\tau) \stackrel{?}{=} H(\tau) \cdot T(\tau)$ 是否成立即可。但这里有一个致命的矛盾：如果把随机点 $\tau$ 明文告诉 Prover，Prover 就可以利用 $\tau$ 现场伪造一个根本不符合 R1CS 约束的多项式，从而作恶。如果不告诉 Prover，Prover 就没办法在不知道 $\tau$ 的情况下算出多项式在 $\tau$ 点的值。

2. Powers of Tau 的诞生：同态加密的“盲算”为了解决这个矛盾，密码学家使用了椭圆曲线上的同态隐藏（Homomorphic Hiding）。我们不能让任何人知道明文 $\tau$，但我们可以把 $\tau$ 的各阶幂次放到椭圆曲线的群上（也就是用基点 $G$ 进行加密）。这就是 Powers of Tau。它是一串加密后的群元素序列：$$[G, \tau \cdot G, \tau^2 \cdot G, \tau^3 \cdot G, \dots, \tau^n \cdot G]$$这里的 $n$ 取决于你的电路规模（即 R1CS 中约束的数量）。如果电路有 $2^{20}$ 条约束，那么就需要生成到 $\tau^{2^{20}}$ 的幂次。它的神奇作用：有了这串加密后的幂次，Prover 虽然不知道明文 $\tau$ 是多少，但如果他手里有一个真实的电路多项式（例如 $P(x) = a + bx + cx^2$），他可以通过同态线性组合，在密文状态下强行算出 $P(\tau) \cdot G$：$$P(\tau) \cdot G = a \cdot [G] + b \cdot [\tau \cdot G] + c \cdot [\tau^2 \cdot G]$$这就实现了“盲算”：Prover 在完全不知道 $\tau$ 的情况下，完成了多项式在 $\tau$ 点的求值，并将这个结果打包成 Proof 发给 Verifier。

3. 在 R1CS 工作流中的具体阶段在实际的工程（如 SnarkJS、Gnark）中，Powers of Tau 的作用体现在两个阶段：

[Phase 1: Powers of Tau (通用)] ───> 产物: 统一的 Tau 幂次密文
                                         │
                                         ▼
[Phase 2: Circuit Specific]    ───> 结合具体的 R1CS 约束
                                         │
                                         ▼
                               产物: Proving Key & Verifying Key
阶段一：Phase 1 (Universal Setup)作用：生成一串足够长的、与任何具体电路都无关的密码学公共参数（即上述的 $\tau$ 的幂次序列）。特点：因为不绑定具体电路，所以它可以被复用。比如社区组织一次跑完能支持 $2^{22}$ 约束的 Powers of Tau 仪式，此后任何人的电路只要约束小于这个规模，都可以直接拿这个产物去用。


阶段二：Phase 2 (Circuit-Specific Setup)作用：把 Phase 1 生成的 Powers of Tau，与你用 Circom 或 Go 写好的具体 R1CS 约束文件结合起来。特点：将通用参数“消化”为你特定电路的 Proving Key（证明密钥）和 Verifying Key（验证密钥）。


4. 为什么叫“可信设置”？$\tau$ 去哪了？生成 Powers of Tau 的过程必须通过 多方安全计算（MPC） 仪式。每个人贡献一部分随机性，计算完后，生成 Powers of Tau 的明文 $\tau$（被称为“有毒废料 / Toxic Waste”）必须被立即彻底销毁。如果明文 $\tau$ 泄露，掌握它的人就可以绕过 R1CS 约束，凭空制造出合法的零知识证明（例如在隐私支付中凭空凭凑出无限的资产）。因此，Powers of Tau 的本质，就是把危险的明文随机数，安全地固化成不可逆的密码学公共基石。


## Non-Interactive
你的直觉非常敏锐！如果你顺着刚才的思路看，“Verifier 发送随机挑战点 $\zeta \rightarrow$ Prover 回应结果和商多项式承诺”，这确实是一个标准的交互式证明（Interactive Proof）。但在实际区块链项目（如以太坊 Layer 2、隐私交易）中，Verifier 通常是一个智能合约。智能合约是死代码，它驻留在链上，没有办法主动、实时地给链下的 Prover 发送一个随机数。如果非要交互，Prover 发起交易 $\rightarrow$ 合约生成随机数 $\rightarrow$ Prover 再发交易，这会带来极高的延迟和高昂的 Gas 费。

为了解决这个问题，现代 ZK 项目引入了一个几乎在所有非交互零知识证明（NIZK）中都会使用的魔法——Fiat-Shamir 变换（Fiat-Shamir Heuristic）。它成功地把需要来回打交道的“交互式系统”，变成了 Prover 一个人就能搞定的“非交互式系统（Non-Interactive）”。1. 核心思想：把 Verifier 替换成一个“绝对公正的哈希函数”在交互式证明中，我们为什么需要 Verifier 给一个随机数？目的是防止 Prover 作弊。随机数必须在 Prover 锁定多项式承诺（第一枚印章）之后生成，这样 Prover 就无法根据随机数去倒推伪造多项式。那如果没有 Verifier，Prover 自己生成这个随机数行不行？如果 Prover 自己随便选一个数字（比如 $x=5$），那他一定会作弊，挑一个对自己最有利、能轻松伪造证明的数字。但如果，我们强迫 Prover 使用一个绝对无法预测、绝对无法控制的算法来生成这个数字呢？这个算法就是密码学哈希函数（如 SHA-256 或 Keccak-256）。


2. 形象化表达：Fiat-Shamir 变换是如何工作的？我们可以把这个变换想象成一个“透明的密封时间胶囊”。第一步（锁定秘密）：Prover 在链下把多项式做成承诺（印章 $C$）。此时，多项式已经被死死锁定了，无法更改。第二步（自制挑战）：Prover 把这个印章 $C$，加上当前电路的其他公开参数，一起扔进哈希函数里：$$\zeta = \text{Hash}(C, \text{公开参数})$$因为哈希函数具有雪崩效应和不可预测性，在印章 $C$ 确定之前，任何人都绝对无法预知 $\zeta$ 的值。这就像 Prover 把印章拍在桌上，大自然（哈希函数）根据这个印章的形状，自动弹出了一个随机数 $\zeta$。第三步（生成证明）：Prover 乖乖拿着这个被迫生成的 $\zeta$，计算出结果 $y = P(\zeta)$ 和商多项式 $H(x)$ 的承诺。最终，Prover 把 [印章 $C$, 结果 $y$, 商多项式印章 $H$] 打包成一个单一的证明（Proof），直接发送给区块链上的 Verifier（智能合约）。

3. Verifier（智能合约）收到后如何验证？当智能合约收到这个 Proof 后，它只需要做两件事：复现挑战点：合约自己在链上把 Proof 里的 [印章 $C$] 拿出来，用同样的哈希函数算一遍：$\zeta = \text{Hash}(C, \text{公开参数})$。因为哈希函数是确定性的，合约算出来的 $\zeta$ 一定和 Prover 链下算出来的一模一样。隔空验算：合约拿着这个 $\zeta$，用我们上一区提到的双线性配对（Pairing）魔法，验证整除关系是否成立。总结：在项目落地时的最终形态通过 Fiat-Shamir 变换，“Verifier 发送随机值”这一步，被哈希函数完美替代了。所以在实际的 ZK 项目中：没有来回的交互。Prover 是在自己的电脑（或高性能服务器）上，独自一人完成了：生成承诺 $\rightarrow$ 哈希计算挑战点 $\rightarrow$ 计算商多项式 $\rightarrow$ 打包 Proof。最终上链的只有一笔带证明的交易。链上的智能合约收到后，闭着眼睛执行一次配对检查，通过了就放行，不通过就拒绝。这就是为什么它被称为 zk-SNARK 中的 N（Non-Interactive，非交互式）。

## Witness

Witness 到底是什么

输入： c <== a * b;

{
  "a": 3,
  "b": 4
}

生成：

witness = [1,3,4,12]

通常：

[常数项, 输入, 中间变量, 输出]

## proof

### proof 实际生成过程

证明者拿：

witness
proving key

计算：

A = g^(A(τ)+α+rδ)
B = g^(B(τ)+β+sδ)
C = ...

这里：

α β δ 是 setup 随机数
r s 是 prover 随机数

最后的结果，proof只是三个椭圆曲线上的点。
当然验证的时候，还需要public的输入，输出，选取的随机数，曲线参数等。

### proof 为什么有随机数

因为：

如果没有：

r,s

proof 会泄露 witness。

随机化后：

同一个 witness：

每次 proof 都不同。

这就是：

Zero-Knowledge

### proof 里面到底有哪些数据

3 个椭圆曲线点。 proof 大小几乎不变。不管多少个constraints
{
  "pi_a": [ax, ay],
  "pi_b": [[bx1,bx2],[by1,by2]],
  "pi_c": [cx, cy]
}