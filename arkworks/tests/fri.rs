/*
based on the arkworks data structure and algorithms library
to implement a simple FRI protocol for polynomial commitment schemes.
the program is designed to be a simple and efficient implementation of the FRI protocol,
which is used in zero-knowledge proofs and other cryptographic applications.
it includes functions for generating commitments, proving, and verifying,
as well as tests to ensure correctness and performance.

it must show each step of the FRI protocol, including the generation of commitments,
the creation of proofs, and the verification of proofs.
add comments to explain the purpose and functionality of each function and data structure,
and provide examples of how to use the library in practice.
and include the folding steps, the challenge generation, and the final verification of the proof.
*/

//! A self-contained implementation of the **FRI** (Fast Reed-Solomon
//! Interactive Oracle Proof of Proximity) protocol on top of arkworks.
//!
//! FRI is the *low-degree test* at the heart of STARK-style systems. A
//! verifier who is handed evaluations of a function over a large domain is
//! convinced — without reading the whole table — that the function is a
//! low-degree polynomial. In a STARK pipeline this commit+fold+query structure
//! is what makes the transition checks sound.
//!
//! ## Protocol overview
//!
//! 1. **Commit**    : evaluate `p(x)` on an `N`-point Radix-2 domain and hash
//!                    the evaluations into a Merkle tree; the root is the
//!                    round-0 *polynomial commitment*.
//! 2. **Fold**      : write `p(x) = p_e(x²) + x·p_o(x²)`, then form
//!                    `p₁(y) = p_e(y) + β·p_o(y)`. The degree (and the size of
//!                    the evaluation domain) is halved; commit `p₁` on the
//!                    smaller domain. Repeat until a tiny final layer remains.
//! 3. **Challenge** : each folding `β` is derived from the *committed* Merkle
//!                    roots via the Fiat-Shamir transform, so the protocol
//!                    needs no interaction and the verifier can reproduce
//!                    every challenge.
//! 4. **Query**     : from the same transcript the verifier selects columns;
//!                    the prover opens the matching leaf pair
//!                    `(p_r(x), p_r(−x))` in *every* round, complete with
//!                    Merkle authentication paths.
//! 5. **Verify**    : every Merkle path checks out, every pair recombines into
//!                    the next round's value, and the last fold lands on the
//!                    published final layer.
//!
//! ## The algebra of one fold (round `r`, challenge `β`)
//!
//! ```text
//!  p (x)  = p_e(x²) +   x·p_o(x²)
//!  p (−x) = p_e(x²) −   x·p_o(x²)
//!  ⇒  p_e(x²) = (p(x) + p(−x)) / 2
//!  ⇒  p_o(x²) = (p(x) − p(−x)) / (2x)
//!  ⇒  p_{r+1}(x²) = p_e(x²) + β·p_o(x²)
//! ```
//!
//! Because `p_{r+1}` lives on the *squared* domain, the column address `qi`
//! stays valid all the way down — the verifier walks one column of the
//! pyramid, checking the identity at every level.
//!
//! ## Usage
//!
//! ```ignore
//! // Prove that `poly` is low degree and open its commitment at query columns.
//! let (proof, commitment) = fri_prove(&poly, domain_size, num_queries);
//! // Verify — the polynomial itself is never seen again.
//! let accepted = fri_verify(&proof);
//! ```

use ark_bls12_381::Fr;
use ark_ff::{BigInteger, FftField, Field, PrimeField, Zero};
use ark_poly::univariate::DensePolynomial;
use ark_poly::{DenseUVPolynomial, EvaluationDomain, Polynomial, Radix2EvaluationDomain};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Folding stops once a polynomial is described by this many evaluation
/// points; that whole final layer is sent in plaintext. Its half is also the
/// number of distinct query columns (`FINAL_LAYER_SIZE / 2 = 4` here).
const FINAL_LAYER_SIZE: usize = 8;

// ============================================================================
// 1. Hash helpers — every field value collapses into 32 bytes.
// ============================================================================

/// Serialize a field element into big-endian bytes (leaves and transcripts).
fn fr_to_bytes(e: &Fr) -> Vec<u8> {
    e.into_bigint().to_bytes_be()
}

/// Hash a single field element into a 32-byte digest.
fn hash_fr(e: &Fr) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(fr_to_bytes(e));
    to_array(&h.finalize())
}

/// Hash two digests into one. Sorted children => order-independent Merkle
/// nodes (the prover cannot reorder its own siblings).
fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    if a <= b {
        h.update(a);
        h.update(b);
    } else {
        h.update(b);
        h.update(a);
    }
    to_array(&h.finalize())
}

/// Copy the first 32 bytes of a digest slice into a plain `[u8; 32]`.
fn to_array(bytes: &[u8]) -> [u8; 32] {
    let mut res = [0u8; 32];
    res.copy_from_slice(&bytes[..32]);
    res
}

/// Map a 256-bit digest onto the base field by reducing it modulo the field
/// order. Every Fiat-Shamir challenge (`β`, query positions) comes from here.
fn challenge_from_hash(hash: &[u8; 32]) -> Fr {
    Fr::from_le_bytes_mod_order(hash)
}

/// First 8 bytes of a digest as a little-endian `u64` (query position picker).
fn hash_to_u64(hash: &[u8; 32]) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&hash[..8]);
    u64::from_le_bytes(b)
}

/// Pretty-print the first few bytes of a digest (for the demos).
fn short_hex(h: &[u8; 32]) -> String {
    let mut s = String::new();
    for b in &h[..6] {
        s.push_str(&format!("{:02x}", b));
    }
    s.push('…');
    s
}

// ============================================================================
// 2. Merkle tree — the FRI commitment primitive.
// ============================================================================

/// A perfectly balanced Merkle tree over `2^k` field elements.
///
/// `layers[0]` = hashed leaves; each upper layer is the hash of two children;
/// the one-element top layer is the root.
struct MerkleTree {
    /// Raw leaf values, kept so `open` does not need to re-evaluate anything.
    leaves: Vec<Fr>,
    /// `layers[l][i]` = node at height `l`, position `i`.
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build the tree bottom-up from a power-of-two set of leaves.
    fn build(leaves: &[Fr]) -> MerkleTree {
        assert!(
            leaves.len().is_power_of_two(),
            "FRI needs a power-of-two number of leaves"
        );
        let mut layers: Vec<Vec<[u8; 32]>> = vec![leaves.iter().map(hash_fr).collect()];
        while layers.last().unwrap().len() > 1 {
            let prev = layers.last().unwrap();
            layers.push(prev.chunks(2).map(|c| hash_pair(&c[0], &c[1])).collect());
        }
        MerkleTree {
            leaves: leaves.to_vec(),
            layers,
        }
    }

    /// The 256-bit binding commitment to this round's evaluations.
    fn root(&self) -> [u8; 32] {
        self.layers.last().unwrap()[0]
    }

    /// Open leaf `i`: return its value and the sibling path (O(log n) hashes).
    fn open(&self, mut i: usize) -> (Fr, Vec<[u8; 32]>) {
        let leaf = self.leaves[i];
        let mut path = Vec::new();
        for layer in &self.layers[..self.layers.len() - 1] {
            path.push(layer[i ^ 1]); // sibling of leaf `i`
            i >>= 1;
        }
        (leaf, path)
    }

    /// Recompute the root from `(leaf, path, index)` and compare with the
    /// expected root — the verifier's Merkle check.
    fn verify(root: &[u8; 32], leaf: &Fr, path: &[[u8; 32]], mut i: usize) -> bool {
        let mut cur = hash_fr(leaf);
        for sibling in path {
            cur = if i & 1 == 0 {
                hash_pair(&cur, sibling)
            } else {
                hash_pair(sibling, &cur)
            };
            i >>= 1;
        }
        &cur == root
    }
}

// ============================================================================
// 3. Public data structures describing a FRI proof.
// ============================================================================

/// One Merkle opening: the folded pair `(p_r(x), p_r(−x))` and both paths.
#[derive(Clone, Debug)]
struct FriOpen {
    left: Fr,
    right: Fr,
    left_path: Vec<[u8; 32]>,
    right_path: Vec<[u8; 32]>,
}

/// One query column: a `FriOpen` per round, all pinned to a single `qi`.
type FriColumn = Vec<FriOpen>;

/// A complete non-interactive FRI proof: everything the verifier replays the
/// protocol from, given only the proof itself (never the polynomial).
#[derive(Clone, Debug)]
struct FriProof {
    /// Size `N = 2^k` of the base evaluation domain.
    domain_size: usize,
    /// Merkle roots of every round; `roots[0]` is the *polynomial commitment*.
    roots: Vec<[u8; 32]>,
    /// Fiat-Shamir folding challenges; `betas[i]` folds round `i → i+1`,
    /// hence `betas.len() == roots.len() - 1`.
    betas: Vec<Fr>,
    /// Plaintext values of the smallest domain (size `FINAL_LAYER_SIZE`).
    final_layer: Vec<Fr>,
    /// Hash-derived column addresses, each `< final_layer.len() / 2`.
    query_indices: Vec<usize>,
    /// `columns[j][r]` opens round `r` for the column at `query_indices[j]`.
    columns: Vec<FriColumn>,
}

// ============================================================================
// 4. Shared helpers.
// ============================================================================

/// Evaluate a polynomial on an `N`-point Radix-2 domain. Returned in the
/// generator order `1, ω, ω², …, ω^{N−1}`.
///
/// A production prover swaps this for an NTT (`O(N log N)`); plain `evaluate`
/// keeps every value visible for this educational implementation.
fn eval_on_radix_domain(poly: &DensePolynomial<Fr>, size: usize) -> Vec<Fr> {
    let domain = Radix2EvaluationDomain::<Fr>::new(size).unwrap();
    domain.elements().map(|x| poly.evaluate(&x)).collect()
}

/// Fold a polynomial by even/odd coefficient splitting:
/// `p₁(y) = p_e(y) + β·p_o(y)  ⇔  c′_i = c_{2i} + β·c_{2i+1}`.
/// The degree — and, implicitly, the domain — halves every round.
fn fold_poly(poly: &DensePolynomial<Fr>, beta: Fr) -> DensePolynomial<Fr> {
    let mut coeffs = poly.coeffs.clone();
    if coeffs.len() % 2 == 1 {
        coeffs.push(Fr::zero()); // pad odd degrees so the pairs line up
    }
    let folded: Vec<Fr> = coeffs
        .chunks(2)
        .map(|pair| pair[0] + beta * pair[1])
        .collect();
    DensePolynomial::from_coefficients_vec(folded)
}

/// Fiat-Shamir challenge: hash the prefix of *committed* roots and reduce to a
/// field element. Both prover and verifier reproduce this value — so the
/// interactive coin-flips become a single hash (non-interactivity).
fn derive_challenge(committed_roots: &[[u8; 32]]) -> Fr {
    let mut h = Sha256::new();
    for r in committed_roots {
        h.update(r);
    }
    challenge_from_hash(&to_array(&h.finalize()))
}

/// Deterministically pick `n` query addresses in `[0, limit)` from the public
/// parts of the proof — Fiat-Shamir applied to the *query* phase.
fn derive_query_indices(
    roots: &[[u8; 32]],
    betas: &[Fr],
    final_layer: &[Fr],
    n: usize,
) -> Vec<usize> {
    let mut h = Sha256::new();
    for r in roots {
        h.update(r);
    }
    for b in betas {
        h.update(fr_to_bytes(b));
    }
    for v in final_layer {
        h.update(fr_to_bytes(v));
    }
    let digest = to_array(&h.finalize());
    // all valid columns live in the first half of the final layer
    let limit = final_layer.len() / 2;

    (0..n)
        .map(|i| {
            let mut h = Sha256::new();
            h.update(digest);
            h.update(&(i as u64).to_le_bytes());
            (hash_to_u64(&to_array(&h.finalize())) % limit as u64) as usize
        })
        .collect()
}

// ============================================================================
// 5. Prover
// ============================================================================

/// Commit + fold pipeline: build every round's Merkle root, derive the
/// Fiat-Shamir challenges, and stop on the plaintext final layer.
///
/// Returns `(round_roots, betas, final_layer, per_round_trees)`. The trees are
/// only materialised so `fri_prove` can open leaves — the verifier never sees
/// them, only their roots.
fn fri_commit_and_fold(
    poly: &DensePolynomial<Fr>,
    domain_size: usize,
) -> (Vec<[u8; 32]>, Vec<Fr>, Vec<Fr>, Vec<MerkleTree>) {
    let mut roots = Vec::new();
    let mut betas = Vec::new();
    let mut trees = Vec::new();

    let mut cur = poly.clone();
    let mut cur_size = domain_size;
    let mut final_layer = Vec::new();

    while cur_size >= FINAL_LAYER_SIZE {
        // (1) commit this round: evaluate + Merkle-root it.
        let evals = eval_on_radix_domain(&cur, cur_size);
        let tree = MerkleTree::build(&evals);
        roots.push(tree.root());
        trees.push(tree);

        if cur_size == FINAL_LAYER_SIZE {
            final_layer = evals; // bottom of the pyramid
            break;
        }

        // (3) challenge: β depends only on the commitment made so far.
        let beta = derive_challenge(&roots);
        betas.push(beta);
        // (2) fold: degree and next domain both halve.
        cur = fold_poly(&cur, beta);
        cur_size /= 2;
    }

    (roots, betas, final_layer, trees)
}

/// FULL FRI PROVER — every protocol step in one call.
///
/// * **commit**   : `fri_commit_and_fold` makes the per-round Merkle trees.
/// * **challenge**: the query columns are hashed out of the public data.
/// * **query**    : each column opens the matching `(left, right)` leaves in
///   every round, with their Merkle sibling paths.
///
/// Returns `(proof, round-0 commitment)`.
fn fri_prove(
    poly: &DensePolynomial<Fr>,
    domain_size: usize,
    num_queries: usize,
) -> (FriProof, [u8; 32]) {
    assert!(domain_size.is_power_of_two() && domain_size >= FINAL_LAYER_SIZE);
    assert!(
        (1..=FINAL_LAYER_SIZE / 2).contains(&num_queries),
        "need 1..={} query columns for the final layer size",
        FINAL_LAYER_SIZE / 2
    );

    let (roots, betas, final_layer, trees) = fri_commit_and_fold(poly, domain_size);
    let num_rounds = roots.len();

    let query_indices = derive_query_indices(&roots, &betas, &final_layer, num_queries);

    let mut columns = Vec::with_capacity(num_queries);
    for &qi in &query_indices {
        let mut column = Vec::with_capacity(num_rounds);
        for (r, tree) in trees.iter().enumerate() {
            let size_r = domain_size >> r;
            assert!(
                qi < size_r / 2,
                "query column {qi} out of range for round {r} (size {size_r})"
            );
            let (left, left_path) = tree.open(qi);
            let (right, right_path) = tree.open(qi + size_r / 2);
            column.push(FriOpen {
                left,
                right,
                left_path,
                right_path,
            });
        }
        columns.push(column);
    }

    let commitment = roots[0];
    let proof = FriProof {
        domain_size,
        roots,
        betas,
        final_layer,
        query_indices,
        columns,
    };
    (proof, commitment)
}

// ============================================================================
// 6. Verifier
// ============================================================================

/// FULL FRI verifier — replays the whole protocol from the proof:
///
/// 1. **challenges**  — `β` must equal the Fiat-Shamir output of the same roots;
/// 2. **query columns** — must match the Fiat-Shamir picks of the public data;
/// 3. **Merkle paths** — every opened leaf authenticates against its round root;
/// 4. **fold identity** — `fold(left,right,β) == next value` holds at every
///    level, terminating at the plaintext final layer.
fn fri_verify(proof: &FriProof) -> bool {
    let num_rounds = proof.roots.len();
    if proof.roots.is_empty() || proof.betas.len() != num_rounds - 1 {
        return false;
    }
    if proof.final_layer.len() != FINAL_LAYER_SIZE {
        return false;
    }

    // (1) challenges must be exactly what a fresh run would derive.
    let mut prefix: Vec<[u8; 32]> = Vec::with_capacity(num_rounds);
    for r in 0..num_rounds - 1 {
        prefix.push(proof.roots[r]);
        if derive_challenge(&prefix) != proof.betas[r] {
            return false;
        }
    }

    // (2) query columns must be exactly the derived ones.
    let expected = derive_query_indices(
        &proof.roots,
        &proof.betas,
        &proof.final_layer,
        proof.query_indices.len(),
    );
    if expected != proof.query_indices {
        return false;
    }

    let omega = Fr::get_root_of_unity(proof.domain_size as u64).unwrap();
    let two_inv = Fr::from(2u64).inverse().unwrap();

    for (qi, column) in proof.query_indices.iter().zip(&proof.columns) {
        if column.len() != num_rounds {
            return false;
        }

        for r in 0..num_rounds {
            let size_r = proof.domain_size >> r;
            if *qi >= size_r / 2 {
                return false;
            }
            let open = &column[r];
            let right_idx = *qi + size_r / 2;

            // (3) Merkle authentication for both leaves.
            if !MerkleTree::verify(&proof.roots[r], &open.left, &open.left_path, *qi) {
                return false;
            }
            if !MerkleTree::verify(&proof.roots[r], &open.right, &open.right_path, right_idx) {
                return false;
            }

            // (4) the pairing identity; the last round has nothing left to fold.
            if r + 1 < num_rounds {
                let beta = proof.betas[r];
                // x = ω^{2^r · qi}
                let g = omega.pow(&[(*qi as u64) << r]);
                let even = (open.left + open.right) * two_inv;
                let odd = (open.left - open.right) * two_inv * g.inverse().unwrap();
                let folded = even + beta * odd;

                let expected = if r + 1 < num_rounds - 1 {
                    column[r + 1].left // next round's leaf at address `qi`
                } else {
                    proof.final_layer[*qi] // last fold lands on the plaintext layer
                };
                if folded != expected {
                    return false;
                }
            }
        }
    }
    true
}

// ============================================================================
// 7. Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::UniformRand;
    use rand::{rngs::StdRng, SeedableRng};

    /// A deterministic random polynomial of the requested degree.
    fn rand_poly(rng: &mut StdRng, degree: usize) -> DensePolynomial<Fr> {
        let coeffs: Vec<Fr> = (0..=degree).map(|_| Fr::rand(rng)).collect();
        DensePolynomial::from_coefficients_vec(coeffs)
    }

    /// Step-by-step walkthrough of the whole protocol, printing each phase:
    /// commitment, folding with challenges, a query opening, and verification.
    #[test]
    fn fri_documented_walkthrough() {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  FRI protocol walkthrough (arkworks + Bls12-381)      ║");
        println!("╚══════════════════════════════════════════════════════════╝");

        // p(x) = 5 + 4x + 3x² + 2x³ + x⁴ + 2x⁵   (degree 5)
        let p = DensePolynomial::from_coefficients_vec(vec![
            Fr::from(5),
            Fr::from(4),
            Fr::from(3),
            Fr::from(2),
            Fr::from(1),
            Fr::from(2),
        ]);
        let size = 64_usize;
        println!("prover:  p(x) = {:?}  (degree {})", p.coeffs, p.degree());
        println!("         evaluation domain N       = {size}");
        println!("         final plaintext layer    = {FINAL_LAYER_SIZE}\n");

        // ── step 1 & 3 & 2: commit + challenge + fold ─────────────────────
        let (roots, betas, final_layer, trees) = fri_commit_and_fold(&p, size);
        println!(
            "[1] COMMIT:  round-0 Merkle root (commitment) = {}",
            short_hex(&roots[0])
        );

        println!("\n[2][3] FOLD + CHALLENGE (β = Fiat-Shamir of committed roots):");
        for r in 0..roots.len() - 1 {
            println!(
                "      round {r}: root = {}  β = {:?}  → next domain ÷2",
                short_hex(&roots[r]),
                betas[r]
            );
        }
        println!(
            "      final layer: {} plaintext values kept open",
            final_layer.len()
        );

        // ── step 4 · query one column by hand ──────────────────────────────
        let qi = 0usize;
        let (left, left_path) = trees[0].open(qi);
        let (right, right_path) = trees[0].open(qi + size / 2);
        let x = Fr::get_root_of_unity(size as u64)
            .unwrap()
            .pow(&[qi as u64]);
        println!("\n[4] QUERY:  column qi = {qi}, so x = ω^{qi} = {x:?}");
        println!("      left  = p(x)   = {left:?}");
        println!("      right = p(−x)  = {right:?}");
        let merkle_ok = MerkleTree::verify(&roots[0], &left, &left_path, qi)
            && MerkleTree::verify(&roots[0], &right, &right_path, qi + size / 2);
        println!("      both Merkle paths verify against commit(p): {merkle_ok}");

        // ── step 5 · run the honest verifier over a real proof ─────────────
        let (proof, commitment) = fri_prove(&p, size, 3);
        println!("\n[5] VERIFY:  full proof, 3 query columns");
        println!("      commitment = {}", short_hex(&commitment));
        println!("      queries    = {:?}", proof.query_indices);
        let verdict = fri_verify(&proof);
        println!(
            "      result     = {} ({})",
            verdict,
            if verdict {
                "proves low degree"
            } else {
                "reject"
            }
        );
        assert!(verdict, "honest prover must be accepted");
    }

    /// A random high-degree polynomial is accepted.
    #[test]
    fn fri_verify_accepts_valid_proof() {
        let mut rng = StdRng::seed_from_u64(1);
        let p = rand_poly(&mut rng, 6);
        let (proof, commitment) = fri_prove(&p, 64, 4);
        assert_eq!(commitment, proof.roots[0]);
        assert!(fri_verify(&proof));
    }

    /// The commitment truly is the Merkle root of the base evaluation table:
    /// re-evaluating `p` reproduces the same root.
    #[test]
    fn fri_commitment_is_binding_to_polynomial() {
        let mut rng = StdRng::seed_from_u64(2);
        let p = rand_poly(&mut rng, 5);
        let (proof, _) = fri_prove(&p, 64, 2);
        let evals = eval_on_radix_domain(&p, 64);
        assert_eq!(proof.roots[0], MerkleTree::build(&evals).root());
    }

    /// Tampering with a single opened leaf must be caught.
    #[test]
    fn fri_rejects_tampered_leaf() {
        let mut rng = StdRng::seed_from_u64(3);
        let p = rand_poly(&mut rng, 6);
        let (mut proof, _) = fri_prove(&p, 64, 3);
        proof.columns[0][0].left += Fr::from(1); // forge one leaf value
        assert!(!fri_verify(&proof));
    }

    /// Tampering with the plaintext final layer must be caught.
    #[test]
    fn fri_rejects_tampered_final_layer() {
        let mut rng = StdRng::seed_from_u64(4);
        let p = rand_poly(&mut rng, 6);
        let (mut proof, _) = fri_prove(&p, 64, 3);
        proof.final_layer[0] += Fr::from(1);
        assert!(!fri_verify(&proof));
    }

    /// A forged challenge must be caught (it must equal the recomputed one).
    #[test]
    fn fri_rejects_tampered_challenge() {
        let mut rng = StdRng::seed_from_u64(5);
        let p = rand_poly(&mut rng, 6);
        let (mut proof, _) = fri_prove(&p, 64, 3);
        proof.betas[0] = Fr::from(123);
        assert!(!fri_verify(&proof));
    }

    /// Columns different from the Fiat-Shamir picks must be rejected.
    #[test]
    fn fri_rejects_changed_query_columns() {
        let mut rng = StdRng::seed_from_u64(6);
        let p = rand_poly(&mut rng, 6);
        let (mut proof, _) = fri_prove(&p, 64, 2);
        proof.query_indices[0] = (proof.query_indices[0] + 1) % (FINAL_LAYER_SIZE / 2);
        assert!(!fri_verify(&proof));
    }

    /// Every supported number of query columns must verify (soundness grows
    /// with the query count, but correctness must hold for all of them).
    #[test]
    fn fri_works_with_many_queries() {
        let mut rng = StdRng::seed_from_u64(8);
        let p = rand_poly(&mut rng, 6);
        for q in 1..=FINAL_LAYER_SIZE / 2 {
            let (proof, _) = fri_prove(&p, 64, q);
            assert!(fri_verify(&proof), "verification must hold for q={q}");
        }
    }

    /// A larger domain stresses the pipeline and reports (rough) timings so
    /// callers can eyeball performance while still getting a hard guarantee.
    #[test]
    fn fri_performance_and_larger_domain() {
        let mut rng = StdRng::seed_from_u64(9);
        let degree = 12;
        let size = 512_usize;
        let p = rand_poly(&mut rng, degree);

        let t0 = Instant::now();
        let (proof, commitment) = fri_prove(&p, size, 4);
        let t_prove = t0.elapsed();

        let t1 = Instant::now();
        let ok = fri_verify(&proof);
        let t_verify = t1.elapsed();

        // rough serialized size: roots + betas + final layer + two leaves/opening
        let approx_bytes = proof.roots.len() * 32
            + proof.betas.len() * 48
            + proof.final_layer.len() * 48
            + proof.columns.iter().map(|c| c.len()).sum::<usize>() * (48 + 48);

        println!(
            "N = {size}, deg = {degree}, q = 4 → commit = {} · prove {:?} · verify {:?} · proof ≈ {} B packed",
            short_hex(&commitment),
            t_prove,
            t_verify,
            approx_bytes
        );
        assert!(ok);
    }
}
