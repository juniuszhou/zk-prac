# zk general

- SNARK
    - circom
    - halo2
    - groth16
    - plank
    - plankish

- STARK
    - 




zk proof 从来不是“验证你输入的是不是对的”，而是：
“验证是否存在某组 witness 满足所有约束。”
witness 并不需要是唯一，而是有还是没有。

比如对于一个简单的加法，任何验证都是通过的，因为你总可以找到二个数相加等于它。
但是对于复杂的约束就不一样了。比如二个约束
a + b = 7
a * b = 7. 那么这个问题就无解了

所以约束的设计非常重要，要求解空间应该非常有限，不容易生成一个合法的witness 或者是output


