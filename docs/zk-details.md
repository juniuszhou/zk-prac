# zk

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
