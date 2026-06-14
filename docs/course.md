# zk
https://securitylab.github.io/cs355-spring25/

https://zkiap.com/



## zkvm
risc zero
sp1

nexus zkVM
Miden STARK based




## 
KZG	多项式 → 单个群元素。因为有一个在setup设置的秘密，因此把所有的参数通过它计算和压缩到一个群元素

STARK  多项式 → Merkle root。没有这个秘密，通过多个点的验证，组成一个merkle 树，得到一个可以验证的证明


                     ┌────────────────────────────┐
                     │        你的程序             │
                     │ (Python / Rust / ML / VM)  │
                     └────────────┬───────────────┘
                                  │
                                  ▼
            ┌────────────────────────────────────┐
            │   CIRCUIT / CONSTRAINT SYSTEM      │
            │                                    │
            │  Circom / Halo2 / PLONK / R1CS     │
            └────────────┬──────────────────────-┘
                         │
     ┌───────────────────┴───────────────────┐
     ▼                                       ▼
┌───────────────┐                   ┌───────────────┐
│    SNARK      │                   │     STARK     │
│ KZG / pairing │                   │ (FRI / hash)  │
│  Groth16/PLONK│                   │               │
└───────┬───────┘                   └───────┬───────┘
        │                                   │
        ▼                                   ▼
   zkVM (SNARK-based)               zkVM (STARK-based)
        │                                   │
        ▼                                   ▼
   zkML (SNARK-ZKML)                 zkML (STARK-ZKML)

