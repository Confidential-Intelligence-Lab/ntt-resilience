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

/// Structural result of ciphertext-ciphertext matrix multiplication.
///
/// For ciphertexts `(B, A)` and `(D, C)`, the unreduced bilinear product
/// contains four matrix products:
///
/// `(B * D, B * C, A * D, A * C)`.
///
/// This type deliberately preserves all four terms. Scheme-specific
/// reduction, relinearization, or key switching is introduced separately.
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixCiphertextProduct<T> {
    bd: MatEcd<T>,
    bc: MatEcd<T>,
    ad: MatEcd<T>,
    ac: MatEcd<T>,
}

impl<T> MatrixCiphertextProduct<T> {
    pub fn new(bd: MatEcd<T>, bc: MatEcd<T>, ad: MatEcd<T>, ac: MatEcd<T>) -> Self {
        assert_eq!(
            bd.rows(),
            bc.rows(),
            "product terms must have matching row counts"
        );
        assert_eq!(
            bd.rows(),
            ad.rows(),
            "product terms must have matching row counts"
        );
        assert_eq!(
            bd.rows(),
            ac.rows(),
            "product terms must have matching row counts"
        );

        assert_eq!(
            bd.cols(),
            bc.cols(),
            "product terms must have matching column counts"
        );
        assert_eq!(
            bd.cols(),
            ad.cols(),
            "product terms must have matching column counts"
        );
        assert_eq!(
            bd.cols(),
            ac.cols(),
            "product terms must have matching column counts"
        );

        assert_eq!(
            bd.batches(),
            bc.batches(),
            "product terms must have matching batch counts"
        );
        assert_eq!(
            bd.batches(),
            ad.batches(),
            "product terms must have matching batch counts"
        );
        assert_eq!(
            bd.batches(),
            ac.batches(),
            "product terms must have matching batch counts"
        );

        Self { bd, bc, ad, ac }
    }

    pub fn bd(&self) -> &MatEcd<T> {
        &self.bd
    }

    pub fn bc(&self) -> &MatEcd<T> {
        &self.bc
    }

    pub fn ad(&self) -> &MatEcd<T> {
        &self.ad
    }

    pub fn ac(&self) -> &MatEcd<T> {
        &self.ac
    }

    pub fn rows(&self) -> usize {
        self.bd.rows()
    }

    pub fn cols(&self) -> usize {
        self.bd.cols()
    }

    pub fn batches(&self) -> usize {
        self.bd.batches()
    }

    /// Consumes the product and returns the four unreduced bilinear terms
    /// `(BD, BC, AD, AC)`.
    pub fn into_terms(self) -> (MatEcd<T>, MatEcd<T>, MatEcd<T>, MatEcd<T>) {
        (self.bd, self.bc, self.ad, self.ac)
    }
}

/// Degree-2 structural ciphertext product produced before relinearization.
///
/// For input ciphertexts `(B, A)` and `(D, C)`, the ciphertext polynomial
///
/// `(B + A*s) * (D + C*s)`
///
/// has coefficients
///
/// `c0 = B*D`
/// `c1 = B*C + A*D`
/// `c2 = A*C`.
///
/// This corresponds to the rank-2 ciphertext consumed by the
/// scheme-specific relinearization stage.
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixCiphertextQuadraticProduct<T> {
    c0: MatEcd<T>,
    c1: MatEcd<T>,
    c2: MatEcd<T>,
}

impl<T> MatrixCiphertextQuadraticProduct<T> {
    pub fn new(c0: MatEcd<T>, c1: MatEcd<T>, c2: MatEcd<T>) -> Self {
        assert_eq!(
            c0.rows(),
            c1.rows(),
            "quadratic product terms must have matching row counts"
        );
        assert_eq!(
            c0.rows(),
            c2.rows(),
            "quadratic product terms must have matching row counts"
        );
        assert_eq!(
            c0.cols(),
            c1.cols(),
            "quadratic product terms must have matching column counts"
        );
        assert_eq!(
            c0.cols(),
            c2.cols(),
            "quadratic product terms must have matching column counts"
        );
        assert_eq!(
            c0.batches(),
            c1.batches(),
            "quadratic product terms must have matching batch counts"
        );
        assert_eq!(
            c0.batches(),
            c2.batches(),
            "quadratic product terms must have matching batch counts"
        );

        Self { c0, c1, c2 }
    }

    pub fn c0(&self) -> &MatEcd<T> {
        &self.c0
    }

    pub fn c1(&self) -> &MatEcd<T> {
        &self.c1
    }

    pub fn c2(&self) -> &MatEcd<T> {
        &self.c2
    }

    pub fn rows(&self) -> usize {
        self.c0.rows()
    }

    pub fn cols(&self) -> usize {
        self.c0.cols()
    }

    pub fn batches(&self) -> usize {
        self.c0.batches()
    }

    pub fn into_terms(self) -> (MatEcd<T>, MatEcd<T>, MatEcd<T>) {
        (self.c0, self.c1, self.c2)
    }
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

/// Scheme-specific reduction of an unreduced ciphertext-ciphertext product.
///
/// Structural CCMM produces four bilinear terms `(BD, BC, AD, AC)`.
/// Implementations of this trait define how those terms are transformed
/// into a valid two-component ciphertext for a particular cryptographic
/// construction.
///
/// No reduction, relinearization, key switching, scaling, or rounding
/// semantics are assumed at this layer.
impl<T> MatrixCiphertextProduct<T>
where
    T: Clone + Default + std::ops::Add<Output = T> + std::ops::Mul<Output = T>,
{
    /// Combines the four bilinear CCMM blocks into the degree-2
    /// ciphertext polynomial coefficients consumed by relinearization.
    ///
    /// For `(BD, BC, AD, AC)`, returns
    ///
    /// `(c0, c1, c2) = (BD, BC + AD, AC)`.
    pub fn combine_degree_two(self) -> MatrixCiphertextQuadraticProduct<T> {
        let (bd, bc, ad, ac) = self.into_terms();
        let middle = bc.add(&ad);

        MatrixCiphertextQuadraticProduct::new(bd, middle, ac)
    }
}

pub trait MatrixCiphertextProductReducer<T> {
    /// Reduces an unreduced four-term product to a two-component ciphertext.
    fn reduce(&self, product: MatrixCiphertextProduct<T>) -> MatrixCiphertext<T>;
}

impl<T> MatrixCiphertextProduct<T> {
    /// Applies a scheme-specific reducer to this unreduced product.
    pub fn reduce_with<R>(self, reducer: &R) -> MatrixCiphertext<T>
    where
        R: MatrixCiphertextProductReducer<T>,
    {
        reducer.reduce(self)
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

    /// Applies structural ciphertext-ciphertext matrix multiplication.
    ///
    /// For ciphertexts `(B, A)` and `(D, C)`, this computes the unreduced
    /// bilinear product
    ///
    /// `(B * D, B * C, A * D, A * C)`.
    ///
    /// The four terms are preserved explicitly. Scheme-specific reduction,
    /// relinearization, and key switching are introduced separately.
    pub fn ccmm(&self, rhs: &Self) -> MatrixCiphertextProduct<T> {
        assert_eq!(
            self.batches(),
            rhs.batches(),
            "ciphertext batch counts must match"
        );
        assert_eq!(
            self.cols(),
            rhs.rows(),
            "ciphertext matrix dimensions must be compatible"
        );

        MatrixCiphertextProduct::new(
            self.b.matmul(&rhs.b),
            self.b.matmul(&rhs.a),
            self.a.matmul(&rhs.b),
            self.a.matmul(&rhs.a),
        )
    }

    /// Applies structural ciphertext-ciphertext matrix multiplication and
    /// immediately dispatches the unreduced product to a scheme-specific
    /// reducer.
    ///
    /// This method does not define reduction semantics. It composes
    /// `ccmm()` with the supplied [`MatrixCiphertextProductReducer`].
    pub fn ccmm_reduce<R>(&self, rhs: &Self, reducer: &R) -> Self
    where
        R: MatrixCiphertextProductReducer<T>,
    {
        self.ccmm(rhs).reduce_with(reducer)
    }

    /// Performs structural CCMM, combines the four bilinear terms into
    /// the degree-2 ciphertext polynomial, and dispatches that product
    /// to a supplied relinearization operation.
    ///
    /// Rescaling is intentionally not part of this operation.
    pub fn ccmm_relinearize<F>(&self, rhs: &Self, relinearize: F) -> Self
    where
        F: FnOnce(MatrixCiphertextQuadraticProduct<T>) -> Self,
    {
        let quadratic = self.ccmm(rhs).combine_degree_two();
        relinearize(quadratic)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn deterministic_ccmm_cross_validation_case() {
        // HEaaN oracle plaintext matrices:
        //
        // X = [[1, 2],      Y = [[5, 6],
        //      [3, 4]]           [7, 8]]
        //
        // X * Y = [[19, 22],
        //          [43, 50]]
        //
        // Split each plaintext matrix into structural ciphertext
        // components such that X = B + A and Y = D + C. This is the
        // ciphertext polynomial evaluated at the test point s = 1.
        //
        // Column-major storage is used throughout.

        let b = filled_encoded(2, 2, 1, &[1_i64, 1, 1, 1]);
        let a = filled_encoded(2, 2, 1, &[0_i64, 2, 1, 3]);

        let d = filled_encoded(2, 2, 1, &[2_i64, 3, 3, 4]);
        let c = filled_encoded(2, 2, 1, &[3_i64, 4, 3, 4]);

        // B + A = [[1, 2], [3, 4]]
        assert_eq!(b.add(&a).raw(), &[1, 3, 2, 4]);

        // D + C = [[5, 6], [7, 8]]
        assert_eq!(d.add(&c).raw(), &[5, 7, 6, 8]);

        let lhs = MatrixCiphertext::new(b, a);
        let rhs = MatrixCiphertext::new(d, c);

        let product = lhs.ccmm(&rhs);

        // Preserve the four independent bilinear blocks.
        let bd = product.bd().clone();
        let bc = product.bc().clone();
        let ad = product.ad().clone();
        let ac = product.ac().clone();

        // Direct expansion at s = 1:
        //
        // BD + BC + AD + AC
        //   = (B + A)(D + C)
        //   = X * Y.
        let expanded = bd.add(&bc).add(&ad).add(&ac);

        assert_eq!(expanded.raw(), &[19, 43, 22, 50]);

        // Now validate the exact degree-2 ciphertext representation.
        let quadratic = product.combine_degree_two();

        assert_eq!(quadratic.c0(), &bd);
        assert_eq!(quadratic.c1(), &bc.add(&ad));
        assert_eq!(quadratic.c2(), &ac);

        // Evaluating c0 + c1*s + c2*s^2 at the same structural
        // test point s = 1 must preserve the matrix product.
        let quadratic_at_one = quadratic.c0().add(quadratic.c1()).add(quadratic.c2());

        assert_eq!(quadratic_at_one.raw(), &[19, 43, 22, 50]);
    }

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
    fn ccmm_expands_all_four_bilinear_terms() {
        // Left ciphertext:
        // B = [[1, 2],
        //      [3, 4]]
        //
        // A = [[5, 6],
        //      [7, 8]]
        //
        // Right ciphertext:
        // D = [[2, 0],
        //      [1, 3]]
        //
        // C = [[4, 1],
        //      [2, 5]]
        //
        // Matrices are stored column-major.
        let b = filled_encoded(2, 2, 1, &[1, 3, 2, 4]);
        let a = filled_encoded(2, 2, 1, &[5, 7, 6, 8]);
        let d = filled_encoded(2, 2, 1, &[2, 1, 0, 3]);
        let c = filled_encoded(2, 2, 1, &[4, 2, 1, 5]);

        let lhs = MatrixCiphertext::new(b, a);
        let rhs = MatrixCiphertext::new(d, c);

        let result = lhs.ccmm(&rhs);

        // B * D = [[4, 6],
        //          [10, 12]]
        let expected_bd = filled_encoded(2, 2, 1, &[4, 10, 6, 12]);

        // B * C = [[8, 11],
        //          [20, 23]]
        let expected_bc = filled_encoded(2, 2, 1, &[8, 20, 11, 23]);

        // A * D = [[16, 18],
        //          [22, 24]]
        let expected_ad = filled_encoded(2, 2, 1, &[16, 22, 18, 24]);

        // A * C = [[32, 35],
        //          [44, 47]]
        let expected_ac = filled_encoded(2, 2, 1, &[32, 44, 35, 47]);

        assert_eq!(result.bd(), &expected_bd);
        assert_eq!(result.bc(), &expected_bc);
        assert_eq!(result.ad(), &expected_ad);
        assert_eq!(result.ac(), &expected_ac);

        assert_eq!(result.rows(), 2);
        assert_eq!(result.cols(), 2);
        assert_eq!(result.batches(), 1);
    }

    #[test]
    #[should_panic(expected = "ciphertext batch counts must match")]
    fn ccmm_rejects_mismatched_batch_counts() {
        let lhs = MatrixCiphertext::new(encoded(2, 2, 1), encoded(2, 2, 1));
        let rhs = MatrixCiphertext::new(encoded(2, 2, 2), encoded(2, 2, 2));

        let _ = lhs.ccmm(&rhs);
    }

    #[test]
    #[should_panic(expected = "ciphertext matrix dimensions must be compatible")]
    fn ccmm_rejects_incompatible_dimensions() {
        let lhs = MatrixCiphertext::new(encoded(2, 3, 1), encoded(2, 3, 1));
        let rhs = MatrixCiphertext::new(encoded(2, 2, 1), encoded(2, 2, 1));

        let _ = lhs.ccmm(&rhs);
    }

    #[test]
    fn ccmm_relinearize_receives_correct_quadratic_product() {
        let lhs = MatrixCiphertext::new(
            filled_encoded(2, 2, 1, &[1, 0, 0, 1]),
            filled_encoded(2, 2, 1, &[2, 0, 0, 2]),
        );

        let rhs = MatrixCiphertext::new(
            filled_encoded(2, 2, 1, &[3, 0, 0, 3]),
            filled_encoded(2, 2, 1, &[4, 0, 0, 4]),
        );

        let result = lhs.ccmm_relinearize(&rhs, |quadratic| {
            // c0 = B*D = 3I
            assert_eq!(quadratic.c0(), &filled_encoded(2, 2, 1, &[3, 0, 0, 3]));

            // c1 = B*C + A*D = 4I + 6I = 10I
            assert_eq!(quadratic.c1(), &filled_encoded(2, 2, 1, &[10, 0, 0, 10]));

            // c2 = A*C = 8I
            assert_eq!(quadratic.c2(), &filled_encoded(2, 2, 1, &[8, 0, 0, 8]));

            // Test-only stand-in for the backend's key-backed relinearizer.
            MatrixCiphertext::new(quadratic.c0().clone(), quadratic.c1().clone())
        });

        assert_eq!(result.b().raw(), &[3, 0, 0, 3]);
        assert_eq!(result.a().raw(), &[10, 0, 0, 10]);
    }

    #[test]
    fn ccmm_reduce_composes_product_and_reducer() {
        struct TestReducer;

        impl MatrixCiphertextProductReducer<i64> for TestReducer {
            fn reduce(&self, product: MatrixCiphertextProduct<i64>) -> MatrixCiphertext<i64> {
                let (bd, _bc, _ad, ac) = product.into_terms();

                // Test-only mapping: verifies operation composition rather
                // than cryptographic reduction semantics.
                MatrixCiphertext::new(bd, ac)
            }
        }

        // B = I, A = 2I
        let lhs = MatrixCiphertext::new(
            filled_encoded(2, 2, 1, &[1, 0, 0, 1]),
            filled_encoded(2, 2, 1, &[2, 0, 0, 2]),
        );

        // D = 3I, C = 4I
        let rhs = MatrixCiphertext::new(
            filled_encoded(2, 2, 1, &[3, 0, 0, 3]),
            filled_encoded(2, 2, 1, &[4, 0, 0, 4]),
        );

        let result = lhs.ccmm_reduce(&rhs, &TestReducer);

        // TestReducer selects BD and AC.
        assert_eq!(result.b(), &filled_encoded(2, 2, 1, &[3, 0, 0, 3]));
        assert_eq!(result.a(), &filled_encoded(2, 2, 1, &[8, 0, 0, 8]));
    }

    #[test]
    fn ccmm_product_combines_into_degree_two_ciphertext() {
        // Four-term product:
        //
        // BD = [[1, 2],
        //       [3, 4]]
        //
        // BC = [[5, 6],
        //       [7, 8]]
        //
        // AD = [[10, 20],
        //       [30, 40]]
        //
        // AC = [[9, 8],
        //       [7, 6]]
        let product = MatrixCiphertextProduct::new(
            filled_encoded(2, 2, 1, &[1, 3, 2, 4]),
            filled_encoded(2, 2, 1, &[5, 7, 6, 8]),
            filled_encoded(2, 2, 1, &[10, 30, 20, 40]),
            filled_encoded(2, 2, 1, &[9, 7, 8, 6]),
        );

        let quadratic = product.combine_degree_two();

        assert_eq!(quadratic.c0(), &filled_encoded(2, 2, 1, &[1, 3, 2, 4]));

        // c1 = BC + AD
        assert_eq!(quadratic.c1(), &filled_encoded(2, 2, 1, &[15, 37, 26, 48]));

        assert_eq!(quadratic.c2(), &filled_encoded(2, 2, 1, &[9, 7, 8, 6]));

        assert_eq!(quadratic.rows(), 2);
        assert_eq!(quadratic.cols(), 2);
        assert_eq!(quadratic.batches(), 1);
    }

    #[test]
    fn product_into_terms_preserves_all_four_terms() {
        let bd = filled_encoded(1, 1, 1, &[1]);
        let bc = filled_encoded(1, 1, 1, &[2]);
        let ad = filled_encoded(1, 1, 1, &[3]);
        let ac = filled_encoded(1, 1, 1, &[4]);

        let product = MatrixCiphertextProduct::new(bd.clone(), bc.clone(), ad.clone(), ac.clone());

        let (actual_bd, actual_bc, actual_ad, actual_ac) = product.into_terms();

        assert_eq!(actual_bd, bd);
        assert_eq!(actual_bc, bc);
        assert_eq!(actual_ad, ad);
        assert_eq!(actual_ac, ac);
    }

    #[test]
    fn product_dispatches_to_scheme_specific_reducer() {
        struct TestReducer;

        impl MatrixCiphertextProductReducer<i64> for TestReducer {
            fn reduce(&self, product: MatrixCiphertextProduct<i64>) -> MatrixCiphertext<i64> {
                let (bd, _bc, _ad, ac) = product.into_terms();

                // This is deliberately a test-only mapping. It validates the
                // reducer boundary, not cryptographic reduction semantics.
                MatrixCiphertext::new(bd, ac)
            }
        }

        let product = MatrixCiphertextProduct::new(
            filled_encoded(1, 1, 1, &[11]),
            filled_encoded(1, 1, 1, &[12]),
            filled_encoded(1, 1, 1, &[13]),
            filled_encoded(1, 1, 1, &[14]),
        );

        let result = product.reduce_with(&TestReducer);

        assert_eq!(result.b(), &filled_encoded(1, 1, 1, &[11]));
        assert_eq!(result.a(), &filled_encoded(1, 1, 1, &[14]));
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
