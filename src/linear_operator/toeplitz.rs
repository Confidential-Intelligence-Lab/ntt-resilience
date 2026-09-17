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
    generator: Option<Vec<SubringPolynomial<T>>>,
}

impl<T> ToeplitzOperator<T> {
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

    /// Constructs the wrapped Toeplitz operator from the decomposition
    /// `(s_0, ..., s_{d-1})` of its generating polynomial.
    ///
    /// # Panics
    ///
    /// Panics if the generator does not contain exactly `d` components.
    pub fn from_generator(d: usize, generator: Vec<SubringPolynomial<T>>) -> Self {
        assert_eq!(
            generator.len(),
            d,
            "Toeplitz generator must contain exactly d subring polynomials"
        );

        Self {
            d,
            blocks: Vec::new(),
            generator: Some(generator),
        }
    }

    pub fn generator(&self) -> Option<&[SubringPolynomial<T>]> {
        self.generator.as_deref()
    }

    /// Number of block rows/columns.
    pub fn d(&self) -> usize {
        self.d
    }

    /// Immutable access to the block polynomials.
    pub fn blocks(&self) -> &[SubringPolynomial<T>] {
        &self.blocks
    }
}

impl<T> ToeplitzOperator<T>
where
    T: Clone
        + Default
        + std::ops::Mul<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + std::ops::SubAssign
        + std::ops::Neg<Output = T>,
{
    /// Applies the operator to a vector of subring polynomials.
    ///
    /// Generator-backed operators use the wrapped Toeplitz action
    ///
    /// `output[i] = sum_j s[(i - j) mod d] * input[j]`,
    ///
    /// with an additional factor of `Y` whenever `j > i`.
    ///
    /// Operators constructed with [`Self::new`] retain the original
    /// block-circulant behavior.
    pub fn apply(&self, input: &[SubringPolynomial<T>]) -> Vec<SubringPolynomial<T>> {
        assert_eq!(
            input.len(),
            self.d,
            "input vector must contain exactly d subring polynomials"
        );

        let source = self.generator.as_deref().unwrap_or(&self.blocks);
        let wrapped = self.generator.is_some();
        let mut output = Vec::with_capacity(self.d);

        for row in 0..self.d {
            let mut accum: Option<SubringPolynomial<T>> = None;

            for (col, polynomial) in input.iter().enumerate() {
                let index = (row + self.d - col) % self.d;
                let mut product = source[index].negacyclic_mul(polynomial);

                if wrapped && col > row {
                    product = product.mul_by_y();
                }

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
    fn generator_constructor_preserves_components() {
        let generator = vec![
            SubringPolynomial::new(vec![1_i64, 2]),
            SubringPolynomial::new(vec![3_i64, 4]),
        ];

        let operator = ToeplitzOperator::from_generator(2, generator.clone());

        assert_eq!(operator.d(), 2);
        assert_eq!(operator.generator(), Some(generator.as_slice()));
    }
    #[test]
    fn generator_action_multiplies_wrapped_terms_by_y() {
        let generator = vec![
            SubringPolynomial::new(vec![1_i64, 0]),
            SubringPolynomial::new(vec![2_i64, 0]),
        ];

        let input = vec![
            SubringPolynomial::new(vec![3_i64, 0]),
            SubringPolynomial::new(vec![5_i64, 0]),
        ];

        let operator = ToeplitzOperator::from_generator(2, generator);
        let output = operator.apply(&input);

        assert_eq!(output[0].coefficients(), &[3, 10]);
        assert_eq!(output[1].coefficients(), &[11, 0]);
    }

    /// Independent reference multiplication in Z[X] / (X^N + 1).
    ///
    /// This deliberately operates on full coefficient vectors rather than
    /// using SubringPolynomial or ToeplitzOperator, so the identity test does
    /// not validate the implementation against itself.
    fn full_ring_negacyclic_mul(lhs: &[i64], rhs: &[i64]) -> Vec<i64> {
        assert_eq!(lhs.len(), rhs.len());

        let n = lhs.len();
        let mut result = vec![0_i64; n];

        for (i, &lhs_coeff) in lhs.iter().enumerate() {
            for (j, &rhs_coeff) in rhs.iter().enumerate() {
                let product = lhs_coeff * rhs_coeff;
                let degree = i + j;

                if degree < n {
                    result[degree] += product;
                } else {
                    result[degree - n] -= product;
                }
            }
        }

        result
    }

    #[test]
    fn toeplitz_vec_identity_matches_full_ring_multiplication() {
        // R = Z[X] / (X^6 + 1), with d = 2, k = 3 and Y = X^2.
        //
        // Both inputs are deliberately dense so that the test exercises:
        //   * multiplication within each R_k component,
        //   * wrapped Toeplitz terms,
        //   * multiplication by Y when a d-boundary is crossed,
        //   * negacyclic reduction in both R_k and the full ring.
        let decomposition = crate::linear_operator::subring::SubringDecomposition::new(2, 3);

        let s = vec![1_i64, -2, 3, 4, -1, 2];
        let a = vec![2_i64, 1, -3, 5, 4, -2];

        let generator = decomposition.decompose(&s);
        let input = decomposition.decompose(&a);

        let operator = ToeplitzOperator::from_generator(decomposition.d(), generator);
        let actual = operator.apply(&input);

        let expected_coefficients = full_ring_negacyclic_mul(&s, &a);
        let expected = decomposition.decompose(&expected_coefficients);

        assert_eq!(actual, expected);
    }
}
