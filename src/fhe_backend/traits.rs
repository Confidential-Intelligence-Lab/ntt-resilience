#![allow(dead_code)]

use num_complex::Complex64;

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
