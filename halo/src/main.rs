// Halo2 import glossary (PLONKish circuit model):
//
// Fp (Fr from bn256)
//   Scalar field element. All witness values and constraint arithmetic live in this
//   finite field (same scalar field as BN254 Groth16 in this repo). Example: 0, 1, 7.
//
// ConstraintSystem<Fp>
//   Passed to `configure()`. Declares columns, gates, and polynomial constraints before
//   any witness is assigned. Think: "circuit blueprint / layout rules".
//
// Column<Advice>
//   Handle to a witness advice column in the trace table. Each cell holds one Fp value
//   at a given row. Private witness data is stored here (e.g. `value`, `is_zero`).
//
// Advice
//   Column kind tag: witness columns written by the prover (as opposed to Fixed or
//   Instance/public columns).
//
// Selector
//   Per-row switch that activates a custom gate. When enabled on a row, the gate's
//   constraint polynomials must evaluate to zero there.
//
// Expression<Fp>
//   Symbolic polynomial over column queries (e.g. s * (1 - value * is_zero)). Gates
//   return `vec![expr]` meaning "this expression must equal zero on selected rows".
//
// Rotation
//   Relative row offset when reading a column inside a gate: Rotation::cur() is the
//   current row, Rotation::next() is the next row, etc. Replaces a raw integer offset.
//
// Layouter<Fp>
//   Used in `synthesize()` / `assign()` to place witness values into regions of the
//   trace table. Handles region allocation and enforces layout consistency.
//
// Value<Fp>
//   Witness value that may be unknown during key generation: `known(x)`, `unknown()`,
//   or mapped with `.map()`. Hides private data from the setup phase; prover fills it
//   later when assigning the region.
//
// Error
//   Result type for circuit configuration and witness assignment failures.
use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
// BN256 scalar field (Fr). Alias `Fp` matches Halo2 book convention for "field prime".
use halo2curves::bn256::Fr as Fp;

#[derive(Debug, Clone)]
pub struct IsZeroConfig {
    pub value: Column<Advice>,
    pub is_zero: Column<Advice>,
    pub selector: Selector,
}

pub struct IsZeroChip {
    config: IsZeroConfig,
}

impl IsZeroChip {
    pub fn construct(config: IsZeroConfig) -> Self {
        IsZeroChip { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> IsZeroConfig {
        let value = meta.advice_column();
        let is_zero = meta.advice_column();
        let selector = meta.selector();

        meta.enable_equality(value);
        meta.enable_equality(is_zero);

        // Custom Gate: (1 - value * inv) * selector = is_zero
        meta.create_gate("is_zero", |meta| {
            let s = meta.query_selector(selector);
            let value = meta.query_advice(value, Rotation::cur());
            let is_zero = meta.query_advice(is_zero, Rotation::cur());

            // 关键：使用 PLONKish 自定义门
            vec![s * (Expression::Constant(Fp::one()) - value * is_zero)]
        });

        IsZeroConfig {
            value,
            is_zero,
            selector,
        }
    }

    pub fn assign(&self, mut layouter: impl Layouter<Fp>, value: Value<Fp>) -> Result<(), Error> {
        layouter.assign_region(
            || "IsZero",
            |mut region| {
                let offset = 0;

                // 开启自定义门
                self.config.selector.enable(&mut region, offset)?;

                // 赋值 value
                region.assign_advice(|| "value", self.config.value, offset, || value)?;

                // 计算 is_zero = value == 0 ? 1 : 0
                let is_zero_val = value.map(|v| {
                    if v == Fp::zero() {
                        Fp::one()
                    } else {
                        Fp::zero()
                    }
                });

                region.assign_advice(|| "is_zero", self.config.is_zero, offset, || is_zero_val)?;

                Ok(())
            },
        )
    }
}

fn main() {
    println!("Hello, world!");
}
