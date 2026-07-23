use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::pasta::Fp;
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector};
use halo2_proofs::poly::Rotation;

/// Proves: I know private `a`, `b` such that `a + b = public c`.
/// Mirrors `circuits/addition.circom` with `component main { public [c] }`.
#[derive(Clone)]
pub struct AddCircuit {
    pub a: Fp,
    pub b: Fp,
    /// Public sum (visible to the verifier via the instance column).
    pub c: Fp,
}

/// Minimum rows = 2^K. K=4 is enough for this tiny circuit.
pub const ADD_CIRCUIT_K: u32 = 4;

#[derive(Clone, Debug)]
pub struct Config {
    a: Column<Advice>,
    b: Column<Advice>,
    /// Private advice cell holding the claimed sum; constrained by the add gate.
    c: Column<Advice>,
    /// Instance column: verifier supplies `c` here (public input / public signal).
    public_c: Column<Instance>,
    /// Activates the add gate only on assigned rows (not on padding rows).
    s_add: Selector,
}

impl Circuit<Fp> for AddCircuit {
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let public_c = meta.instance_column();
        let s_add = meta.selector();

        // Required so advice cell `c` can be equality-constrained to `public_c`.
        meta.enable_equality(c);
        meta.enable_equality(public_c);

        meta.create_gate("add gate", |meta| {
            let s = meta.query_selector(s_add);
            let a_val = meta.query_advice(a, Rotation::cur());
            let b_val = meta.query_advice(b, Rotation::cur());
            let c_val = meta.query_advice(c, Rotation::cur());

            vec![s * (a_val + b_val - c_val)]
        });

        Config {
            a,
            b,
            c,
            public_c,
            s_add,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // Assign private witness into advice columns; return the `c` cell handle.
        let c_cell: AssignedCell<Fp, Fp> = layouter.assign_region(
            || "add",
            |mut region| {
                config.s_add.enable(&mut region, 0)?;

                region.assign_advice(|| "a", config.a, 0, || Value::known(self.a))?;
                region.assign_advice(|| "b", config.b, 0, || Value::known(self.b))?;
                region.assign_advice(|| "c", config.c, 0, || Value::known(self.c))
            },
        )?;

        // Expose `c` as a public input: instance[row] must equal advice cell `c`.
        // The verifier only sees the instance column value (e.g. 7), not `a` or `b`.
        layouter.constrain_instance(c_cell.cell(), config.public_c, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::dev::MockProver;

    #[test]
    fn valid_addition_with_public_c() {
        let circuit = AddCircuit {
            a: Fp::from(3),
            b: Fp::from(4),
            c: Fp::from(7),
        };
        let public_inputs = vec![vec![Fp::from(7)]];
        let prover = MockProver::run(ADD_CIRCUIT_K, &circuit, public_inputs).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    #[test]
    fn rejects_wrong_public_c() {
        let circuit = AddCircuit {
            a: Fp::from(3),
            b: Fp::from(4),
            c: Fp::from(7),
        };
        let public_inputs = vec![vec![Fp::from(8)]];
        let prover = MockProver::run(ADD_CIRCUIT_K, &circuit, public_inputs).unwrap();
        assert!(prover.verify().is_err());
    }
}
