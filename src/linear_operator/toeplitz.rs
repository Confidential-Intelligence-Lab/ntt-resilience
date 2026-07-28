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

        Self { d, blocks }
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
}
