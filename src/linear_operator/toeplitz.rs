#![cfg_attr(not(test), allow(dead_code))]

use super::subring::SubringPolynomial;

/// Toeplitz operator over the subring R_k.
///
/// This represents the linear operator
///
///     Toep_k^d(s)
///
/// described in the Crypto 2026 paper.
///
/// The application logic is introduced in a later patch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToeplitzOperator<T> {
    d: usize,
    blocks: Vec<SubringPolynomial<T>>,
    generator: Option<SubringPolynomial<T>>,
}

impl<T> ToeplitzOperator<T>
where
    T: Clone
        + Default
        + std::ops::Mul<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + std::ops::SubAssign,
{
    /// Constructs a Toeplitz operator from its block representation.
    ///
    /// # Panics
    ///
    /// Panics if the number of blocks is not equal to `d`.
    pub fn new(d: usize, blocks: Vec<SubringPolynomial<T>>) -> Self {
        assert_eq!(
            blocks.len(),
            d,
            "Toeplitz operator must contain exactly d blocks"
        );

        Self {
            d,
            blocks,
            generator: None,
        }
    }

    pub fn from_generator(d: usize, generator: SubringPolynomial<T>) -> Self {
        Self {
            d,
            blocks: Vec::new(),
            generator: Some(generator),
        }
    }

    pub fn generator(&self) -> Option<&SubringPolynomial<T>> {
        self.generator.as_ref()
    }

    /// Number of block rows/columns.
    pub fn d(&self) -> usize {
        self.d
    }

    /// Immutable access to the block polynomials.
    pub fn blocks(&self) -> &[SubringPolynomial<T>] {
        &self.blocks
    }

    /// Applies the Toeplitz operator to a vector of subring polynomials.
    ///
    /// The operator is interpreted as a block-circulant matrix whose first
    /// column is given by `self.blocks`.
    pub fn apply(&self, input: &[SubringPolynomial<T>]) -> Vec<SubringPolynomial<T>> {
        assert_eq!(
            input.len(),
            self.d,
            "input vector must contain exactly d subring polynomials"
        );

        let mut output = Vec::with_capacity(self.d);

        for row in 0..self.d {
            let mut accum: Option<SubringPolynomial<T>> = None;

            for (col, polynomial) in input.iter().enumerate() {
                let block = &self.blocks[(row + self.d - col) % self.d];
                let product = block.negacyclic_mul(polynomial);

                accum = Some(match accum {
                    Some(sum) => sum.add(&product),
                    None => product,
                });
            }

            output.push(accum.expect("d must be positive"));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear_operator::subring::SubringPolynomial;

    #[test]
    fn reports_dimensions() {
        let blocks = vec![
            SubringPolynomial::new(vec![1u64, 2]),
            SubringPolynomial::new(vec![3, 4]),
            SubringPolynomial::new(vec![5, 6]),
        ];

        let toep = ToeplitzOperator::new(3, blocks);

        assert_eq!(toep.d(), 3);
        assert_eq!(toep.blocks().len(), 3);
    }

    #[test]
    #[should_panic]
    fn rejects_wrong_number_of_blocks() {
        let blocks = vec![SubringPolynomial::new(vec![1u64, 2])];

        let _ = ToeplitzOperator::new(3, blocks);
    }

    #[test]
    fn identity_operator_returns_input() {
        let one = SubringPolynomial::new(vec![1_i64, 0]);
        let zero = SubringPolynomial::new(vec![0_i64, 0]);

        let operator = ToeplitzOperator::new(3, vec![one.clone(), zero.clone(), zero.clone()]);

        let input = vec![
            SubringPolynomial::new(vec![2_i64, 3]),
            SubringPolynomial::new(vec![4_i64, 5]),
            SubringPolynomial::new(vec![6_i64, 7]),
        ];

        let output = operator.apply(&input);

        assert_eq!(output, input);
    }

    #[test]
    fn generator_constructor_preserves_polynomial() {
        let poly = SubringPolynomial::new(vec![1_i64, 2, 3]);

        let toep = ToeplitzOperator::from_generator(4, poly.clone());

        assert_eq!(toep.d(), 4);
        assert_eq!(toep.generator(), Some(&poly));
    }
}
