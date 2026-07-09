use ark_bls12_381::Fr;
use ark_ff::{FftField, Field, One, UniformRand, Zero};
use rand::{rngs::StdRng, SeedableRng};

fn bit_reverse(mut x: usize, log_n: u32) -> usize {
    let mut r = 0;
    for _ in 0..log_n {
        r = (r << 1) | (x & 1);
        x >>= 1;
    }
    r
}

fn fft(coeffs: &[Fr], omega: Fr) -> Vec<Fr> {
    let n = coeffs.len();
    assert!(n.is_power_of_two());
    let log_n = n.trailing_zeros();

    let mut a = coeffs.to_vec();
    for i in 0..n {
        let j = bit_reverse(i, log_n);
        if i < j {
            a.swap(i, j);
        }
    }

    let mut len = 2;
    while len <= n {
        let wlen = omega.pow(&[(n / len) as u64]);
        for i in (0..n).step_by(len) {
            let mut w = Fr::one();
            for j in 0..len / 2 {
                let u = a[i + j];
                let v = a[i + j + len / 2] * w;
                a[i + j] = u + v;
                a[i + j + len / 2] = u - v;
                w *= wlen;
            }
        }
        len *= 2;
    }
    a
}

fn ifft(evals: &[Fr], omega: Fr) -> Vec<Fr> {
    let n = evals.len();
    let omega_inv = omega.inverse().unwrap();
    let mut coeffs = fft(evals, omega_inv);
    let n_inv = Fr::from(n as u64).inverse().unwrap();
    for c in coeffs.iter_mut() {
        *c *= n_inv;
    }
    coeffs
}

#[test]
fn test_fft_roundtrip() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for log_n in 0..=5 {
        let n = 1 << log_n;
        let omega = <Fr as FftField>::get_root_of_unity(n).unwrap();

        // generate random coefficients
        let coeffs: Vec<Fr> = (0..n).map(|_| Fr::rand(&mut rng)).collect();
        // perform FFT and then IFFT
        let evals = fft(&coeffs, omega);
        let recovered = ifft(&evals, omega);

        assert_eq!(coeffs, recovered, "FFT round-trip failed for n={}", n);
    }
}

#[test]
fn test_fft_evaluation() {
    let mut rng = StdRng::seed_from_u64(1u64);
    let omega = <Fr as FftField>::get_root_of_unity(8).unwrap();
    let n = 8_usize;

    let coeffs: Vec<Fr> = (0..n).map(|_| Fr::rand(&mut rng)).collect();
    let evals = fft(&coeffs, omega);

    let mut omega_k = Fr::one();
    for k in 0..n {
        let mut naive = Fr::zero();
        let mut x_pow = Fr::one();
        for c in &coeffs {
            naive += *c * x_pow;
            x_pow *= omega_k;
        }
        assert_eq!(evals[k], naive, "Mismatch at index {}", k);
        omega_k *= omega;
    }
}
