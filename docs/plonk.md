# Plonk

PLONK 和 Plonkish 经常被同时提及，但它们的含义所处的维度不同：PLONK 是一个具体的证明系统（协议），
而 Plonkish 是一种电路构建的范式（Arithmetization/代数化方法）。


在原始的 PLONK 论文中，电路的每一行被严格限制为“三点式标准门”。
每一行只能有 2 个输入 Advice 列（左 $a$, 右 $b$）和 1 个输出 Advice 列（$c$）。

Plonkish：自定义门 + 跨行引用 (Custom Gates & Multi-row)
自定义门可以让每个自定义门可以包含多个输入，但是在实践中，收到计算的限制。
在主流框架（如 Halo2）中，为了保证证明器（Prover）的性能，框架通常会限制一个 Gate 的 Degree 不能超过 5（或者 9，取决于具体的后端配置）。如果你的输入太多且它们相互绑定做乘法，导致 Degree 爆炸，Prover 在做多项式做 FFT（快速傅里叶变换）时的计算开销和内存占用就会急剧飙升


Plonkish 打破了原始 PLONK 的限制。它允许电路设计者自由定义表格的列数，并且允许跨行引用数据。
Lookup Tables（查找表）的支持。

## 
在 PLONK / Halo2 里：

✔ 一个 step（row）里可以放：多个变量 + 多个算术操作 + 多个中间关系

❗ 但前提是：这些操作必须能写成“一个 gate 约束”


## 
Table = 数据
Gate  = 规则

数据 → Witness Polynomial
规则 → Constraint Polynomial

最后组合成一个 Quotient Polynomial


Halo2 的真实设计方式
Halo2 的 step 不是“固定粒度”，而是：
✔ 一个 row = 一个 “instruction bundle”

这个 bundle 里可以包含：

✔ advice columns（witness）
✔ fixed columns（常量）
✔ instance columns（public input）
✔ selector（控制 gate）


## 
设计 trade-off（核心）
策略	结果
少 step（复杂 gate）	constraint少，但 gate复杂
多 step（简单 gate）	constraint多，但 gate简单



## witness 长什么样

先给最核心结论

在 PLONK / Halo2 里：

witness = entire execution trace（整张表）

也就是：

witness = 所有 rows + 所有 columns 的值