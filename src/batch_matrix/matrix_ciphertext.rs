#![allow(dead_code)]

use super::mat_ecd::MatEcd;

/// Structural ciphertext representation for encrypted batch matrices.
///
/// The ciphertext consists of two matrix components `(B, A)`. At this layer
/// the type only enforces that both components have identical logical
/// dimensions. Cryptographic operations are introduced separately.
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixCiphertext<T> {
    b: MatEcd<T>,
    a: MatEcd<T>,
}

impl<T> MatrixCiphertext<T> {
    /// Constructs a matrix ciphertext from its `(B, A)` components.
    ///
    /// # Panics
    ///
    /// Panics if the two components have different matrix or batch
    /// dimensions.
    pub fn new(b: MatEcd<T>, a: MatEcd<T>) -> Self {
        assert_eq!(
            b.rows(),
            a.rows(),
            "ciphertext components must have matching row counts"
        );
        assert_eq!(
            b.cols(),
            a.cols(),
            "ciphertext components must have matching column counts"
        );
        assert_eq!(
            b.batches(),
            a.batches(),
            "ciphertext components must have matching batch counts"
        );

        Self { b, a }
    }

    /// Returns the `B` component.
    pub fn b(&self) -> &MatEcd<T> {
        &self.b
    }

    /// Returns the `A` component.
    pub fn a(&self) -> &MatEcd<T> {
        &self.a
    }

    /// Number of matrix rows in each ciphertext component.
    pub fn rows(&self) -> usize {
        self.b.rows()
    }

    /// Number of matrix columns in each ciphertext component.
    pub fn cols(&self) -> usize {
        self.b.cols()
    }

    /// Number of batched matrices represented by the ciphertext.
    pub fn batches(&self) -> usize {
        self.b.batches()
    }

    /// Consumes the ciphertext and returns `(B, A)`.
    pub fn into_components(self) -> (MatEcd<T>, MatEcd<T>) {
        (self.b, self.a)
    }
}

impl<T> MatrixCiphertext<T>
where
    T: Clone + Default + std::ops::Add<Output = T> + std::ops::Mul<Output = T>,
{
    /// Applies ciphertext-plaintext matrix multiplication component-wise.
    ///
    /// For ciphertext `(B, A)` and plaintext matrix `U`, this computes
    ///
    /// `(B * U, A * U)`.
    ///
    /// This is the structural CPMM operation before scale management and
    /// backend-specific ciphertext handling are introduced.
    pub fn cpmm(&self, plaintext: &MatEcd<T>) -> Self {
        assert_eq!(
            self.b.batches(),
            plaintext.batches(),
            "ciphertext and plaintext batch counts must match"
        );
        assert_eq!(
            self.b.cols(),
            plaintext.rows(),
            "ciphertext and plaintext matrix dimensions must be compatible"
        );

        Self::new(self.b.matmul(plaintext), self.a.matmul(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch_matrix::real_matrix::BatchMatrix;

    fn encoded(rows: usize, cols: usize, batches: usize) -> MatEcd<i64> {
        MatEcd::encode(&BatchMatrix::new(rows, cols, batches))
    }

    fn filled_encoded(rows: usize, cols: usize, batches: usize, values: &[i64]) -> MatEcd<i64> {
        assert_eq!(values.len(), rows * cols * batches);

        let mut matrix = BatchMatrix::new(rows, cols, batches);

        for batch in 0..batches {
            for col in 0..cols {
                for row in 0..rows {
                    let index = batch * rows * cols + row + col * rows;
                    matrix.set(batch, row, col, values[index]);
                }
            }
        }

        MatEcd::encode(&matrix)
    }

    #[test]
    fn cpmm_multiplies_both_ciphertext_components() {
        // B = [[1, 2],
        //      [3, 4]]
        //
        // A = [[5, 6],
        //      [7, 8]]
        //
        // U = [[2, 1],
        //      [0, 3]]
        //
        // Matrices are stored column-major.
        let b = filled_encoded(2, 2, 1, &[1, 3, 2, 4]);

        let a = filled_encoded(2, 2, 1, &[5, 7, 6, 8]);

        let u = filled_encoded(2, 2, 1, &[2, 0, 1, 3]);

        let ciphertext = MatrixCiphertext::new(b, a);
        let result = ciphertext.cpmm(&u);

        let expected_b = filled_encoded(2, 2, 1, &[2, 6, 7, 15]);

        let expected_a = filled_encoded(2, 2, 1, &[10, 14, 23, 31]);

        assert_eq!(result.b(), &expected_b);
        assert_eq!(result.a(), &expected_a);
    }

    #[test]
    #[should_panic(expected = "batch counts must match")]
    fn cpmm_rejects_mismatched_batch_counts() {
        let ciphertext = MatrixCiphertext::new(encoded(2, 2, 1), encoded(2, 2, 1));

        let plaintext = encoded(2, 2, 2);

        let _ = ciphertext.cpmm(&plaintext);
    }

    #[test]
    #[should_panic(expected = "matrix dimensions must be compatible")]
    fn cpmm_rejects_incompatible_dimensions() {
        let ciphertext = MatrixCiphertext::new(encoded(2, 3, 1), encoded(2, 3, 1));

        let plaintext = encoded(2, 2, 1);

        let _ = ciphertext.cpmm(&plaintext);
    }

    #[test]
    fn constructor_preserves_components_and_dimensions() {
        let b = encoded(2, 3, 4);
        let a = encoded(2, 3, 4);

        let ciphertext = MatrixCiphertext::new(b.clone(), a.clone());

        assert_eq!(ciphertext.b(), &b);
        assert_eq!(ciphertext.a(), &a);
        assert_eq!(ciphertext.rows(), 2);
        assert_eq!(ciphertext.cols(), 3);
        assert_eq!(ciphertext.batches(), 4);
    }

    #[test]
    fn into_components_returns_owned_components() {
        let b = encoded(2, 2, 1);
        let a = encoded(2, 2, 1);

        let ciphertext = MatrixCiphertext::new(b.clone(), a.clone());
        let (actual_b, actual_a) = ciphertext.into_components();

        assert_eq!(actual_b, b);
        assert_eq!(actual_a, a);
    }

    #[test]
    #[should_panic(expected = "matching row counts")]
    fn rejects_mismatched_rows() {
        let b = encoded(2, 3, 1);
        let a = encoded(3, 3, 1);

        let _ = MatrixCiphertext::new(b, a);
    }

    #[test]
    #[should_panic(expected = "matching column counts")]
    fn rejects_mismatched_columns() {
        let b = encoded(2, 3, 1);
        let a = encoded(2, 4, 1);

        let _ = MatrixCiphertext::new(b, a);
    }

    #[test]
    #[should_panic(expected = "matching batch counts")]
    fn rejects_mismatched_batches() {
        let b = encoded(2, 3, 1);
        let a = encoded(2, 3, 2);

        let _ = MatrixCiphertext::new(b, a);
    }
}
