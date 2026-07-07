#![allow(dead_code)]

use num_complex::Complex64;

use crate::fhe_backend::traits::FheBackend;

#[derive(Debug, Clone, PartialEq)]
pub struct ToyPlaintext {
    values: Vec<Complex64>,
}

impl ToyPlaintext {
    pub fn values(&self) -> &[Complex64] {
        &self.values
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToyCiphertext {
    values: Vec<Complex64>,
}

impl ToyCiphertext {
    pub fn values(&self) -> &[Complex64] {
        &self.values
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToyBackend;

impl FheBackend for ToyBackend {
    type Plaintext = ToyPlaintext;
    type Ciphertext = ToyCiphertext;

    fn encode(&self, values: &[Complex64]) -> Self::Plaintext {
        ToyPlaintext {
            values: values.to_vec(),
        }
    }

    fn decode(&self, pt: &Self::Plaintext) -> Vec<Complex64> {
        pt.values.clone()
    }

    fn encrypt(&self, pt: &Self::Plaintext) -> Self::Ciphertext {
        ToyCiphertext {
            values: pt.values.clone(),
        }
    }

    fn decrypt(&self, ct: &Self::Ciphertext) -> Self::Plaintext {
        ToyPlaintext {
            values: ct.values.clone(),
        }
    }

    fn add(&self, a: &Self::Ciphertext, b: &Self::Ciphertext) -> Self::Ciphertext {
        assert_eq!(a.values.len(), b.values.len(), "ciphertext length mismatch");

        ToyCiphertext {
            values: a
                .values
                .iter()
                .zip(&b.values)
                .map(|(x, y)| *x + *y)
                .collect(),
        }
    }

    fn mul(&self, a: &Self::Ciphertext, b: &Self::Ciphertext) -> Self::Ciphertext {
        assert_eq!(a.values.len(), b.values.len(), "ciphertext length mismatch");

        ToyCiphertext {
            values: a
                .values
                .iter()
                .zip(&b.values)
                .map(|(x, y)| *x * *y)
                .collect(),
        }
    }

    fn rotate(&self, a: &Self::Ciphertext, steps: isize) -> Self::Ciphertext {
        let n = a.values.len();
        if n == 0 {
            return ToyCiphertext { values: Vec::new() };
        }

        let mut out = vec![Complex64::new(0.0, 0.0); n];

        for (i, value) in out.iter_mut().enumerate() {
            let src = (i as isize + steps).rem_euclid(n as isize) as usize;
            *value = a.values[src];
        }

        ToyCiphertext { values: out }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toy_backend_roundtrip_works() {
        let backend = ToyBackend;
        let values = vec![Complex64::new(1.0, 2.0), Complex64::new(3.0, 4.0)];

        let pt = backend.encode(&values);
        let ct = backend.encrypt(&pt);
        let decoded = backend.decode(&backend.decrypt(&ct));

        assert_eq!(decoded, values);
    }

    #[test]
    fn toy_backend_add_mul_rotate_work() {
        let backend = ToyBackend;

        let a = backend.encrypt(&backend.encode(&[
            Complex64::new(1.0, 0.0),
            Complex64::new(2.0, 0.0),
            Complex64::new(3.0, 0.0),
        ]));

        let b = backend.encrypt(&backend.encode(&[
            Complex64::new(10.0, 0.0),
            Complex64::new(20.0, 0.0),
            Complex64::new(30.0, 0.0),
        ]));

        let sum = backend.add(&a, &b);
        assert_eq!(sum.values()[0], Complex64::new(11.0, 0.0));

        let prod = backend.mul(&a, &b);
        assert_eq!(prod.values()[2], Complex64::new(90.0, 0.0));

        let rot = backend.rotate(&a, 1);
        assert_eq!(
            rot.values(),
            &[
                Complex64::new(2.0, 0.0),
                Complex64::new(3.0, 0.0),
                Complex64::new(1.0, 0.0)
            ]
        );
    }
}
