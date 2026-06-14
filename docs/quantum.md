# quantum

## why quantum shor 可以快速破解 ECC / RSA
因为它们是基于prime 分解，通常一个个去式要很长时间。
但是shor是通过找到周期（period）来破解

## 
ML-KEM 
lattice crypto
PQ signatures

基于lattice的加密体系不是通过prime来构建，所以不会被quantum攻破

Kyber/ML-KEM 用于加密/KEM，Dilithium/ML-DSA 用于签名

Module-Lattice-Based ML
DSA Digital Singaure Algorithm
KEM Key Encapsulation Mechanism

主流算法：CRYSTALS-Kyber（ML-KEM）


参数集（安全级别）公钥大小密文大小密钥生成封装（加密）解封装（解密）
Kyber-512 (Level 1)~800 B~768 B~0.01 ms~0.01 ms~0.008 ms
Kyber-768 (Level 3)~1,184 B~1,088 B~0.02 ms~0.02 ms~0.02 ms
Kyber-1024 (Level 5)~1,568 B~1,568 B~0.03 ms~0.03 ms~0.03 ms

主流算法：CRYSTALS-Dilithium（ML-DSA）（推荐主选）

参数集公钥大小签名大小密钥生成签名生成验证
Dilithium-2 (Level 2)~1.3 KB~2.4 KB~0.7 ms~0.7 ms~0.1 ms
Dilithium-5 (Level 5)~2.6 KB~4.6 KB~1-2 ms~1-2 ms~0.2 ms

时间与空间复杂度总结




操作时间复杂度（渐近）实际性能（现代 CPU）空间开销与经典算法对比
密钥生成O(n² log q) 或更好（NTT）微秒 ~ 毫秒级中等（几 KB）比 RSA 快
加密/封装O(n² log q)0.01 ~ 0.05 ms1-3 KB（密钥+密文）比 ECC 慢 2-5x
签名生成O(n² log q) + 拒绝采样0.5 ~ 2 ms2-5 KB（签名）比 ECDSA 慢
验证O(n² log q)0.1 ~ 0.3 ms-与 ECDSA 相当或更快

n：维度（通常 256~1024），q：模数（Kyber ~3329，Dilithium ~8380417）。
NTT 加速：把多项式乘法从 O(n²) 降到 O(n log n)，是性能关键。
硬件友好：非常适合 FPGA、ASIC 加速，吞吐量可达极高（GPU 上每秒百万次签名）。