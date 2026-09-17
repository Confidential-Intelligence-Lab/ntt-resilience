#![allow(dead_code)]

use super::real_matrix::BatchMatrix;

/// Plaintext matrix encoding used by the batch CCMM layer.
///
/// `MatEcd` preserves the logical `(batch, row, column)` dimensions while
/// storing the encoded values in the same batch-major, column-major order as
/// `BatchMatrix`:
///
/// `batch * rows * cols + row + col * rows`.
///
/// This type intentionally models only the matrix-layout encoding. CKKS slot
/// packing and ciphertext representation are separate layers.
#[derive(Debug, Clone, PartialEq)]
pub struct MatEcd<T> {
    rows: usize,
    cols: usize,
    batches: usize,
    data: Vec<T>,
}

impl<T: Clone> MatEcd<T> {
    /// Encodes a batch matrix without changing coefficient values.
    pub fn encode(matrix: &BatchMatrix<T>) -> Self {
        assert!(matrix.rows() > 0, "matrix must contain at least one row");
        assert!(matrix.cols() > 0, "matrix must contain at least one column");
        assert!(
            matrix.batches() > 0,
            "matrix must contain at least one batch"
        );

        Self {
            rows: matrix.rows(),
            cols: matrix.cols(),
            batches: matrix.batches(),
            data: matrix.raw().to_vec(),
        }
    }

    /// Decodes the representation back into a batch matrix.
    pub fn decode(&self) -> BatchMatrix<T>
    where
        T: Default,
    {
        let mut matrix = BatchMatrix::new(self.rows, self.cols, self.batches);

        for batch in 0..self.batches {
            for col in 0..self.cols {
                for row in 0..self.rows {
                    matrix.set(batch, row, col, self.get(batch, row, col).clone());
                }
            }
        }

        matrix
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn batches(&self) -> usize {
        self.batches
    }

    pub fn raw(&self) -> &[T] {
        &self.data
    }

    pub fn get(&self, batch: usize, row: usize, col: usize) -> &T {
        let index = self.index(batch, row, col);
        &self.data[index]
    }

    fn index(&self, batch: usize, row: usize, col: usize) -> usize {
        assert!(batch < self.batches, "batch index out of bounds");
        assert!(row < self.rows, "row index out of bounds");
        assert!(col < self.cols, "column index out of bounds");

        batch * self.rows * self.cols + row + col * self.rows
    }
}

impl<T> MatEcd<T>
where
    T: Clone + Default + std::ops::Add<Output = T> + std::ops::Mul<Output = T>,
{
    /// Multiplies corresponding matrices in each batch.
    ///
    /// If `self` has shape `(batch, m, n)` and `rhs` has shape
    /// `(batch, n, p)`, the result has shape `(batch, m, p)`.
    ///
    /// # Panics
    ///
    /// Panics if the batch counts differ or the inner matrix dimensions
    /// are incompatible.
    pub fn matmul(&self, rhs: &Self) -> Self {
        assert_eq!(
            self.batches, rhs.batches,
            "batch counts must match for matrix multiplication"
        );
        assert_eq!(self.cols, rhs.rows, "inner matrix dimensions must match");

        let mut data = vec![T::default(); self.batches * self.rows * rhs.cols];

        for batch in 0..self.batches {
            for col in 0..rhs.cols {
                for row in 0..self.rows {
                    let mut sum = T::default();

                    for inner in 0..self.cols {
                        sum = sum
                            + self.get(batch, row, inner).clone()
                                * rhs.get(batch, inner, col).clone();
                    }

                    let index = batch * self.rows * rhs.cols + row + col * self.rows;
                    data[index] = sum;
                }
            }
        }

        Self {
            rows: self.rows,
            cols: rhs.cols,
            batches: self.batches,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_batch_matmul(lhs: &BatchMatrix<i64>, rhs: &BatchMatrix<i64>) -> BatchMatrix<i64> {
        assert_eq!(lhs.batches(), rhs.batches());
        assert_eq!(lhs.cols(), rhs.rows());

        let mut out = BatchMatrix::new(lhs.rows(), rhs.cols(), lhs.batches());

        for batch in 0..lhs.batches() {
            for col in 0..rhs.cols() {
                for row in 0..lhs.rows() {
                    let mut sum = 0_i64;

                    for inner in 0..lhs.cols() {
                        sum += *lhs.get(batch, row, inner) * *rhs.get(batch, inner, col);
                    }

                    out.set(batch, row, col, sum);
                }
            }
        }

        out
    }

    #[test]
    fn encoded_batch_matmul_matches_independent_reference() {
        // Two batches of 2x3 multiplied by 3x2 matrices.
        let mut lhs = BatchMatrix::<i64>::new(2, 3, 2);
        let mut rhs = BatchMatrix::<i64>::new(3, 2, 2);

        let lhs_values = [
            // batch 0, column-major
            1, 4, 2, 5, 3, 6, // batch 1
            -1, 2, 3, -4, 5, 6,
        ];

        let rhs_values = [
            // batch 0, column-major
            7, 9, 11, 8, 10, 12, // batch 1
            2, -1, 3, 4, 5, -2,
        ];

        for batch in 0..2 {
            for col in 0..3 {
                for row in 0..2 {
                    let index = batch * 6 + row + col * 2;
                    lhs.set(batch, row, col, lhs_values[index]);
                }
            }
        }

        for batch in 0..2 {
            for col in 0..2 {
                for row in 0..3 {
                    let index = batch * 6 + row + col * 3;
                    rhs.set(batch, row, col, rhs_values[index]);
                }
            }
        }

        let expected = reference_batch_matmul(&lhs, &rhs);

        let encoded_lhs = MatEcd::encode(&lhs);
        let encoded_rhs = MatEcd::encode(&rhs);
        let encoded_product = encoded_lhs.matmul(&encoded_rhs);
        let actual = encoded_product.decode();

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "batch counts must match")]
    fn matmul_rejects_mismatched_batch_counts() {
        let lhs = MatEcd::encode(&BatchMatrix::<i64>::new(2, 2, 1));
        let rhs = MatEcd::encode(&BatchMatrix::<i64>::new(2, 2, 2));

        let _ = lhs.matmul(&rhs);
    }

    #[test]
    #[should_panic(expected = "inner matrix dimensions must match")]
    fn matmul_rejects_incompatible_dimensions() {
        let lhs = MatEcd::encode(&BatchMatrix::<i64>::new(2, 3, 1));
        let rhs = MatEcd::encode(&BatchMatrix::<i64>::new(2, 2, 1));

        let _ = lhs.matmul(&rhs);
    }

    #[test]
    fn encode_preserves_dimensions_and_layout() {
        let mut matrix = BatchMatrix::<i64>::new(2, 3, 2);

        let mut value = 1_i64;
        for batch in 0..2 {
            for col in 0..3 {
                for row in 0..2 {
                    matrix.set(batch, row, col, value);
                    value += 1;
                }
            }
        }

        let encoded = MatEcd::encode(&matrix);

        assert_eq!(encoded.rows(), 2);
        assert_eq!(encoded.cols(), 3);
        assert_eq!(encoded.batches(), 2);
        assert_eq!(encoded.raw(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

        assert_eq!(*encoded.get(0, 0, 0), 1);
        assert_eq!(*encoded.get(0, 1, 2), 6);
        assert_eq!(*encoded.get(1, 0, 0), 7);
        assert_eq!(*encoded.get(1, 1, 2), 12);
    }

    #[test]
    fn decode_inverts_encode() {
        let mut matrix = BatchMatrix::<i64>::new(2, 2, 2);

        matrix.set(0, 0, 0, 1);
        matrix.set(0, 1, 0, -2);
        matrix.set(0, 0, 1, 3);
        matrix.set(0, 1, 1, 4);

        matrix.set(1, 0, 0, -5);
        matrix.set(1, 1, 0, 6);
        matrix.set(1, 0, 1, 7);
        matrix.set(1, 1, 1, -8);

        let encoded = MatEcd::encode(&matrix);
        let decoded = encoded.decode();

        assert_eq!(decoded, matrix);
    }

    #[test]
    fn encoding_supports_generic_values() {
        let mut matrix = BatchMatrix::<u64>::new(1, 2, 1);
        matrix.set(0, 0, 0, 42);
        matrix.set(0, 0, 1, 99);

        let encoded = MatEcd::encode(&matrix);
        let decoded = encoded.decode();

        assert_eq!(decoded, matrix);
    }

    #[test]
    #[should_panic(expected = "matrix must contain at least one row")]
    fn rejects_zero_rows() {
        let matrix = BatchMatrix::<i64>::new(0, 2, 1);
        let _ = MatEcd::encode(&matrix);
    }

    #[test]
    #[should_panic(expected = "matrix must contain at least one column")]
    fn rejects_zero_columns() {
        let matrix = BatchMatrix::<i64>::new(2, 0, 1);
        let _ = MatEcd::encode(&matrix);
    }

    #[test]
    #[should_panic(expected = "matrix must contain at least one batch")]
    fn rejects_zero_batches() {
        let matrix = BatchMatrix::<i64>::new(2, 2, 0);
        let _ = MatEcd::encode(&matrix);
    }
}
