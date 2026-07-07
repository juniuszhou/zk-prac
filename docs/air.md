# AIR Algebraic Intermediate Representation

## air and plonky3
AIR 是“图纸”（设计规范）： 它规定了你如何把一个计算问题（比如 RISC-V CPU 的运行逻辑）翻译成数学多项式和约束条件。

Plonky3 是“工厂/工具箱”（实现框架）： 它提供了一整套密码学组件（哈希函数、多项式承诺、PIOP 协议等），用来把这份“图纸”真正变成一个可执行的、极速的 ZK 证明系统。

## air 工作原理

AIR 通过“执行轨迹表（Trace Table）”和“相邻行约束（Transition Constraints）”来工作。



## air 过程
代码中声明 Trace Table 有多少列，以及定义行与行之间的代数约束

Plonky3 拿到你的 AIR 之后，利用它内置的 STARK 协议（基于多项式交互式Oracle论证，即 PIOP），将这些 AIR 约束转化为多项式

Plonky3 调用你配置的 PCS（多项式承诺方案，如 FRI 或 Jagged PCS），对这些多项式进行“承诺（Commit）”并生成最终的 STARK 证明。


LogUp / GKR 
由于一个复杂的虚拟机不能只用一张巨大的 AIR 表（否则内存会爆炸），SP1 会把系统拆分成几十个独立的芯片（Chips），比如内存芯片、乘法芯片、CPU芯片。

每一个芯片，本质上都是一个独立的 AIR。

Plonky3 提供了跨 AIR 的连接纽带： Plonky3 实现了高效的 LogUp 查找论证（Lookup Argument）。它允许这几十个不同的 AIR 表之间进行“通信”和数据校验


## Fibonacci 函数的air 表示
它只有二列数据

- 边界约束  (Boundary Constraints)
    在第 0 行，第一列必须为 0：$$col_0[0] = 0 \implies col_0[0] = 0$$
    
    在第 0 行，第二列必须为 1：$$col_1[0] = 1 \implies col_1[0] - 1 = 0$$

- 状态转移约束 (Transition Constraints) x 代表行号

C_1(x) = col_0[i+1] - col_1[i] = 0$$$$

C_2(x) = col_1[i+1] - (col_0[i] + col_1[i]) = 0$$

## AIR 的电路需要手工写吗

真相是：无论是 AIR 还是 Plonkish，数学意义上的“底层电路（约束）”都需要有人手写。 你之所以觉得“AIR 可以根据 Rust 代码自动生成”，是因为像 SP1 或 RISC Zero 这样的项目，有一群顶级密码学工程师已经帮你把最痛苦、最底层、最通用的那套“CPU 模拟电路”用 AIR 写好了。

通用的 CPU 约束： SP1 的官方团队在底层写了一套永恒不变的、通用的 RISC-V CPU AIR 电路。

SP1 的底层是由无数个纯手写、高度优化的 AIR 芯片电路 拼接而成的。他们把 RISC-V 规范里的每一条标准指令，都变成了数学上的约束。

他们写好的底层 AIR 电路主要包含以下几大核心板块：

解码芯片 (Decode Chip)： 一套 AIR 约束，用来证明：“读进来的这串 32 位二进制数据，的确是一条合法的 RISC-V ADD 指令，而不是别的。”

算术芯片 (ALU Chip)： 一套 AIR 约束，专门负责证明整数加减法（ADD/SUB）、位运算（AND/OR/XOR）的数学正确性。

内存芯片 (Memory Chip)： 极其复杂的一套 AIR 约束，利用 LogUp 查找论证，用来证明：“如果在时钟周期 10 往内存地址 0x1000 写入了 42，那么在时钟周期 15 读取该地址时，读出来的绝对不能是别的值。”

这套电路在发布时就已经固定死了（通过编译变成了可执行的 Prover 源码）。它不关心你的业务逻辑是什么，它只认 RISC-V 指令。

