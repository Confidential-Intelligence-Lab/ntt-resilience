#![allow(dead_code)]

use num_complex::Complex64;

use crate::batch_matrix::matrix_ciphertext::{MatrixCiphertext, MatrixCiphertextQuadraticProduct};

/// Backend capability for relinearizing a degree-2 matrix ciphertext.
///
/// Structural CCMM produces coefficients `(c0, c1, c2)` representing
///
/// `c0 + c1*s + c2*s^2`.
///
/// A concrete FHE backend uses its scheme-specific multiplication/evaluation
/// key material to transform that quadratic ciphertext into the ordinary
/// two-component ciphertext representation.
///
/// Rescaling is intentionally separate from this capability.
pub trait MatrixRelinearizer<T> {
    fn relinearize_matrix(
        &self,
        product: MatrixCiphertextQuadraticProduct<T>,
    ) -> MatrixCiphertext<T>;
}

pub trait FheBackend {
    type Plaintext;
    type Ciphertext;

    fn encode(&self, values: &[Complex64]) -> Self::Plaintext;
    fn decode(&self, pt: &Self::Plaintext) -> Vec<Complex64>;

    fn encrypt(&self, pt: &Self::Plaintext) -> Self::Ciphertext;
    fn decrypt(&self, ct: &Self::Ciphertext) -> Self::Plaintext;

    fn add(&self, a: &Self::Ciphertext, b: &Self::Ciphertext) -> Self::Ciphertext;
    fn mul(&self, a: &Self::Ciphertext, b: &Self::Ciphertext) -> Self::Ciphertext;
    fn rotate(&self, a: &Self::Ciphertext, steps: isize) -> Self::Ciphertext;
}
