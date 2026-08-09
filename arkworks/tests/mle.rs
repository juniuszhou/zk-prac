use ark_bn254::Fr;
use ark_ff::{One, Zero};

/// A tiny multilinear extension (MLE) example over a boolean hypercube.
///
/// For a vector of values `v[0..2^n)`, the MLE at a point `r` is the weighted
/// sum over all boolean inputs. This is the same primitive used by many ZK
/// protocols when a table is represented as a multilinear polynomial.
///
/// value 是代表一个gate的取值，point 是一个挑战值，mle_eval 就是计算这个gate的多项式在挑战点的取值
/// 这里计算的是point是一个n维向量，values是一个长度为2^n的向量，代表了在布尔超立方体上所有点的函数值。mle_eval函数计算的是这个多项式在point点的值。
pub fn mle_eval(values: &[Fr], point: &[Fr]) -> Fr {
    println!("mle_eval: point length is {:?}", point.len());
    assert_eq!(values.len(), 1usize << point.len());

    let mut acc = Fr::zero();
    for index in 0..values.len() {
        let mut term = Fr::one();
        for (bit_index, challenge) in point.iter().enumerate() {
            let bit = ((index >> bit_index) & 1) == 1;

            // if the bit is 1, we take the challenge; if the bit is 0, we take (1 - challenge)
            let basis = if bit {
                *challenge
            } else {
                Fr::one() - *challenge
            };
            term *= basis;
        }
        acc += values[index] * term;
    }
    acc
}

/// A very small sum-check example for the polynomial
///   g(x1, x2) = x1 * x2 + x1 + 2 * x2 + 3
///
/// The prover claims that the sum over all boolean assignments is 19. The
/// sum-check protocol reduces this to one univariate check per variable.
pub fn run_sum_check_demo() -> (Fr, Fr, Fr) {
    let poly = |x1: Fr, x2: Fr| -> Fr { x1 * x2 + x1 + Fr::from(2u64) * x2 + Fr::from(3u64) };

    let sum = poly(Fr::zero(), Fr::zero())
        + poly(Fr::zero(), Fr::one())
        + poly(Fr::one(), Fr::zero())
        + poly(Fr::one(), Fr::one());

    // Round 1: sum over x2 and keep the remaining polynomial in x1.
    let round1_poly = |x1: Fr| -> Fr { poly(x1, Fr::zero()) + poly(x1, Fr::one()) };
    let round1_challenge = Fr::from(7u64);
    let round1_value = round1_poly(round1_challenge);

    // Round 2: now evaluate the remaining one-variable polynomial at a new point.
    // 现在x1已经被固定为round1_challenge，剩下的就是一个关于x2的一元多项式了.接下来就是设置一个新的挑战点round2_challenge，然后计算这个一元多项式在这个点的值round2_value
    let round2_poly = |x2: Fr| -> Fr { poly(round1_challenge, x2) };
    let round2_challenge = Fr::from(11u64);
    let round2_value = round2_poly(round2_challenge);

    (sum, round1_value, round2_value)
}

/// A tiny GKR-style example.
///
/// GKR checks a layered circuit by recursively reducing each layer to a sum-check
/// problem. For this toy circuit we only have one addition gate:
///   output = left + right
///
/// The function shows the same high-level ideas: values are represented by MLEs,
/// and the gate relation is checked with a small sum-check style reduction.
pub fn run_gkr_demo() -> Fr {
    let layer0 = vec![Fr::from(2u64), Fr::from(3u64)];
    let output = layer0[0] + layer0[1];
    let claimed_output = Fr::from(5u64);
    assert_eq!(output, claimed_output);

    // The verifier can ask for an MLE evaluation on the input layer. In this
    // educational example we use a single challenge and compare it with the
    // expected affine form.
    let point = vec![Fr::from(7u64)];
    let mle_value = mle_eval(&layer0, &point);
    let expected_mle = Fr::from(2u64) + point[0];
    assert_eq!(mle_value, expected_mle);

    // A real GKR proof would recurse over layers; here we just show the same
    // sum-check reduction that would be used inside the layer consistency check.
    let (_sum, _round1_value, _round2_value) = run_sum_check_demo();

    output
}

/// The setup phase creates the public parameters used by both prover and verifier.
///
/// In a real protocol, these parameters would be derived from the circuit shape.
/// Here we keep them simple: one challenge per layer and the expected layer sizes.
#[derive(Debug, Clone)]
pub struct CircuitSetup {
    pub challenges: Vec<Fr>,
    pub layer_sizes: Vec<usize>,
}

impl CircuitSetup {
    pub fn new() -> Self {
        Self {
            challenges: vec![Fr::from(3u64), Fr::from(5u64), Fr::from(7u64)],
            layer_sizes: vec![2, 2, 2],
        }
    }
}

/// Public data for the verifier.
///
/// The verifier does not need the private witness, but it does need the public
/// claims about the circuit layers and the final output.
#[derive(Debug, Clone)]
pub struct PublicData {
    pub layer_values: Vec<Vec<Fr>>,
    pub expected_layer_outputs: Vec<Fr>,
    pub final_output: Fr,
}

/// The proof transcript is intentionally tiny.
///
/// Each layer proof stores the values that the prover would send during the
/// reduction: an MLE evaluation and a small sum-check value.
#[derive(Debug, Clone)]
pub struct LayerProof {
    pub output_claim: Fr,
    pub mle_value: Fr,
    pub sum_check_value: Fr,
}

#[derive(Debug, Clone)]
pub struct MultiLayerProof {
    pub layer_proofs: Vec<LayerProof>,
    pub final_output: Fr,
}

/// Build a simple three-layer circuit with two values per layer:
///   layer 0: [2, 3]
///   layer 1: [5, 7]      // each value is a simple aggregation of the layer above
///   layer 2: [12, 14]    // the final layer aggregates the prior layer
///
/// This is a toy GKR-style circuit with multiple layers and a clear recursion.
pub fn setup_multi_layer_demo() -> (CircuitSetup, PublicData, Vec<Vec<Fr>>) {
    let layer0 = vec![Fr::from(2u64), Fr::from(3u64)];
    let layer1 = vec![Fr::from(5u64), Fr::from(7u64)];
    let layer2 = vec![Fr::from(12u64), Fr::from(14u64)];

    let layers = vec![layer0, layer1, layer2.clone()];
    let expected_layer_outputs = vec![
        layers[1].iter().fold(Fr::zero(), |acc, value| acc + *value),
        layers[2].iter().fold(Fr::zero(), |acc, value| acc + *value),
    ];
    let public_data = PublicData {
        layer_values: layers.clone(),
        expected_layer_outputs: expected_layer_outputs.clone(),
        final_output: layers[2].iter().fold(Fr::zero(), |acc, value| acc + *value),
    };

    (CircuitSetup::new(), public_data, layers)
}

/// Generate a proof for the multi-layer example.
///
/// The prover uses the setup parameters and the private witness values to build
/// a small proof transcript. In a full system this transcript would be much more
/// complex, but the structure is the same: one reduction per layer.
pub fn generate_multi_layer_proof(setup: &CircuitSetup, layers: &[Vec<Fr>]) -> MultiLayerProof {
    assert_eq!(setup.layer_sizes.len(), layers.len());

    let mut layer_proofs = Vec::with_capacity(layers.len());
    for (index, layer) in layers.iter().enumerate() {
        let challenge = setup.challenges[index];
        let point = vec![challenge];
        let mle_value = mle_eval(layer, &point);

        // For this toy example, the sum-check round checks that the layer values
        // are consistent with the next layer output claim.
        let output_claim = if index + 1 < layers.len() {
            layers[index + 1]
                .iter()
                .fold(Fr::zero(), |acc, value| acc + *value)
        } else {
            layer.iter().fold(Fr::zero(), |acc, value| acc + *value)
        };
        let sum_check_value =
            layer.iter().fold(Fr::zero(), |acc, value| acc + *value) + output_claim;

        layer_proofs.push(LayerProof {
            output_claim,
            mle_value,
            sum_check_value,
        });
    }

    MultiLayerProof {
        layer_proofs,
        final_output: layers
            .last()
            .unwrap()
            .iter()
            .fold(Fr::zero(), |acc, value| acc + *value),
    }
}

/// Verify a proof using the public data and setup parameters.
///
/// This is a simplified verifier. It recomputes the MLE and the toy sum-check
/// values from the public data and ensures that the proof is consistent.
pub fn verify_multi_layer_proof(
    setup: &CircuitSetup,
    public_data: &PublicData,
    proof: &MultiLayerProof,
) -> bool {
    if proof.layer_proofs.len() != setup.layer_sizes.len() {
        return false;
    }

    for (index, layer_proof) in proof.layer_proofs.iter().enumerate() {
        let layer = &public_data.layer_values[index];
        let challenge = setup.challenges[index];
        let point = vec![challenge];
        let expected_mle = mle_eval(layer, &point);
        if layer_proof.mle_value != expected_mle {
            return false;
        }

        let expected_output_claim = if index + 1 < public_data.layer_values.len() {
            public_data.layer_values[index + 1]
                .iter()
                .fold(Fr::zero(), |acc, value| acc + *value)
        } else {
            layer.iter().fold(Fr::zero(), |acc, value| acc + *value)
        };
        if layer_proof.output_claim != expected_output_claim {
            return false;
        }

        let expected_sum_check =
            layer.iter().fold(Fr::zero(), |acc, value| acc + *value) + layer_proof.output_claim;
        if layer_proof.sum_check_value != expected_sum_check {
            return false;
        }

        if index < public_data.expected_layer_outputs.len()
            && layer_proof.output_claim != public_data.expected_layer_outputs[index]
        {
            return false;
        }
    }

    proof.final_output == public_data.final_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mle_eval_matches_the_expected_affine_form() {
        let values = vec![Fr::from(2u64), Fr::from(3u64)];
        let point = vec![Fr::from(7u64)];

        let actual = mle_eval(&values, &point);
        let expected = Fr::from(2u64) + point[0];

        /*
        For a 1-variable multilinear extension (MLE), the boolean hypercube has 2 points:
        f(0) = 2 (index 0 of values) and f(1) = 3 (index 1).

        The MLE formula is linear interpolation:
        f̃(x) = f(0) · (1 - x) + f(1) · x
                = 2 · (1 - x) + 3 · x
                = 2 - 2x + 3x
                = 2 + x


        根据下面的这个公式，我们就可以理解 mle_eval 的算法是什么样的
        For a 2-variable MLE (values at f(0,0), f(0,1), f(1,0), f(1,1)):
            f̃(x₁, x₂) = f(0,0)·(1−x₁)(1−x₂) + f(0,1)·(1−x₁)x₂ + f(1,0)·x₁(1−x₂) + f(1,1)·x₁·x₂
        */
        assert_eq!(actual, expected);
    }

    #[test]
    fn sum_check_demo_matches_the_small_polynomial() {
        let (sum, round1_value, round2_value) = run_sum_check_demo();

        assert_eq!(sum, Fr::from(19u64));
        assert_eq!(round1_value, Fr::from(29u64));
        assert_eq!(round2_value, Fr::from(109u64));
    }

    #[test]
    fn gkr_demo_checks_a_tiny_gate() {
        let output = run_gkr_demo();
        assert_eq!(output, Fr::from(5u64));
    }

    #[test]
    fn multi_layer_proof_verifies_for_a_valid_circuit() {
        let (setup, public_data, layers) = setup_multi_layer_demo();
        let proof = generate_multi_layer_proof(&setup, &layers);

        assert!(verify_multi_layer_proof(&setup, &public_data, &proof));
    }

    #[test]
    fn tampered_proof_is_rejected() {
        let (setup, public_data, layers) = setup_multi_layer_demo();
        let mut proof = generate_multi_layer_proof(&setup, &layers);
        proof.final_output += Fr::one();

        assert!(!verify_multi_layer_proof(&setup, &public_data, &proof));
    }
}
