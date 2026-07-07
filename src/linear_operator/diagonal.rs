#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::batch_matrix::real_matrix::Vector;
use crate::linear_operator::rotation::{Rotation, RotationPlan};

use num_complex::Complex64;

use crate::fhe_backend::traits::FheBackend;

#[derive(Debug, Clone, PartialEq)]
pub struct DiagonalOperator<T> {
    dimension: usize,
    diagonals: BTreeMap<isize, Vector<T>>,
}

impl<T> DiagonalOperator<T> {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            diagonals: BTreeMap::new(),
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn insert_diagonal(&mut self, rotation: isize, values: Vector<T>) {
        assert_eq!(
            values.size(),
            self.dimension,
            "diagonal length must match operator dimension"
        );
        self.diagonals.insert(rotation, values);
    }

    pub fn diagonals(&self) -> &BTreeMap<isize, Vector<T>> {
        &self.diagonals
    }

    pub fn rotation_plan(&self) -> RotationPlan {
        RotationPlan::new(self.diagonals.keys().copied().map(Rotation).collect())
    }
}

impl<T> DiagonalOperator<T>
where
    T: Clone + Default + std::ops::AddAssign + std::ops::Mul<Output = T>,
{
    pub fn apply_plain(&self, input: &[T]) -> Vec<T> {
        assert_eq!(
            input.len(),
            self.dimension,
            "input length must match operator dimension"
        );

        let mut output = vec![T::default(); self.dimension];

        for (&rot, diag) in &self.diagonals {
            for (i, out_i) in output.iter_mut().enumerate() {
                let src = rotated_index(i, rot, self.dimension);
                *out_i += diag.get(i).clone() * input[src].clone();
            }
        }

        output
    }
}

impl DiagonalOperator<Complex64> {
    pub fn apply_backend<B>(&self, backend: &B, input: &B::Ciphertext) -> B::Ciphertext
    where
        B: FheBackend,
        B::Plaintext: Clone,
        B::Ciphertext: Clone,
    {
        let mut acc: Option<B::Ciphertext> = None;

        for (&rot, diag) in &self.diagonals {
            let rotated = backend.rotate(input, rot);
            let mask = backend.encode(diag.raw());
            let mask_ct = backend.encrypt(&mask);
            let term = backend.mul(&rotated, &mask_ct);

            acc = Some(match acc {
                Some(existing) => backend.add(&existing, &term),
                None => term,
            });
        }

        acc.expect("diagonal operator must contain at least one diagonal")
    }
}

fn rotated_index(i: usize, rotation: isize, n: usize) -> usize {
    let n = n as isize;
    let idx = (i as isize + rotation).rem_euclid(n);
    idx as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fhe_backend::toy_backend::ToyBackend;

    #[test]
    fn diagonal_operator_identity_works() {
        let mut op = DiagonalOperator::new(3);
        op.insert_diagonal(0, Vector::from_vec(vec![1.0, 1.0, 1.0]));

        let out = op.apply_plain(&[2.0, 3.0, 4.0]);
        assert_eq!(out, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn diagonal_operator_rotation_works() {
        let mut op = DiagonalOperator::new(3);
        op.insert_diagonal(1, Vector::from_vec(vec![1.0, 1.0, 1.0]));

        let out = op.apply_plain(&[10.0, 20.0, 30.0]);
        assert_eq!(out, vec![20.0, 30.0, 10.0]);
    }

    #[test]
    fn rotation_plan_contains_diagonal_keys() {
        let mut op = DiagonalOperator::<f64>::new(4);
        op.insert_diagonal(-1, Vector::from_vec(vec![1.0; 4]));
        op.insert_diagonal(2, Vector::from_vec(vec![1.0; 4]));

        let rotations: Vec<_> = op.rotation_plan().rotations().iter().map(|r| r.0).collect();
        assert_eq!(rotations, vec![-1, 2]);
    }

    #[test]
    fn diagonal_operator_applies_through_toy_backend() {
        let backend = ToyBackend;

        let mut op = DiagonalOperator::new(3);
        op.insert_diagonal(
            0,
            Vector::from_vec(vec![
                Complex64::new(2.0, 0.0),
                Complex64::new(2.0, 0.0),
                Complex64::new(2.0, 0.0),
            ]),
        );
        op.insert_diagonal(
            1,
            Vector::from_vec(vec![
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0),
            ]),
        );

        let input = backend.encrypt(&backend.encode(&[
            Complex64::new(10.0, 0.0),
            Complex64::new(20.0, 0.0),
            Complex64::new(30.0, 0.0),
        ]));

        let out = op.apply_backend(&backend, &input);

        assert_eq!(
            out.values(),
            &[
                Complex64::new(40.0, 0.0), // 2*10 + 20
                Complex64::new(70.0, 0.0), // 2*20 + 30
                Complex64::new(70.0, 0.0), // 2*30 + 10
            ]
        );
    }
}
