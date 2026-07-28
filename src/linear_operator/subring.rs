//! Subring decomposition implementing the paper's Vec_k^d mapping.
//!
//! This module is foundational infrastructure and is not yet used by the
//! executable. Remove this allowance once the Toeplitz layer consumes it.

#![cfg_attr(not(test), allow(dead_code))]

/// A polynomial in the negacyclic subring
///
/// `R_k = Z[Y] / (Y^k + 1)`.
///
/// Coefficients are stored in ascending degree order:
///
/// `coeffs[i]` is the coefficient of `Y^i`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubringPolynomial<T> {
    coeffs: Vec<T>,
}

impl<T> SubringPolynomial<T> {
    /// Creates a subring polynomial from coefficients in ascending degree
    /// order.
    ///
    /// # Panics
    ///
    /// Panics when `coeffs` is empty.
    pub fn new(coeffs: Vec<T>) -> Self {
        assert!(
            !coeffs.is_empty(),
            "a subring polynomial must contain at least one coefficient"
        );

        Self { coeffs }
    }

    /// Returns the subring degree parameter `k`.
    pub fn k(&self) -> usize {
        self.coeffs.len()
    }

    /// Returns the coefficients in ascending degree order.
    pub fn coefficients(&self) -> &[T] {
        &self.coeffs
    }

    /// Consumes the polynomial and returns its coefficient vector.
    pub fn into_coefficients(self) -> Vec<T> {
        self.coeffs
    }
}

impl<T> SubringPolynomial<T>
where
    T: Clone,
{
    /// Adds two subring polynomials coefficient-wise.
    ///
    /// # Panics
    ///
    /// Panics when the two polynomials have different values of `k`.
    pub fn add(&self, rhs: &Self) -> Self
    where
        T: std::ops::Add<Output = T>,
    {
        self.assert_same_k(rhs);

        let coeffs = self
            .coeffs
            .iter()
            .cloned()
            .zip(rhs.coeffs.iter().cloned())
            .map(|(lhs, rhs)| lhs + rhs)
            .collect();

        Self::new(coeffs)
    }

    /// Subtracts two subring polynomials coefficient-wise.
    ///
    /// # Panics
    ///
    /// Panics when the two polynomials have different values of `k`.
    pub fn sub(&self, rhs: &Self) -> Self
    where
        T: std::ops::Sub<Output = T>,
    {
        self.assert_same_k(rhs);

        let coeffs = self
            .coeffs
            .iter()
            .cloned()
            .zip(rhs.coeffs.iter().cloned())
            .map(|(lhs, rhs)| lhs - rhs)
            .collect();

        Self::new(coeffs)
    }

    fn assert_same_k(&self, rhs: &Self) {
        assert_eq!(
            self.k(),
            rhs.k(),
            "subring polynomials must have the same value of k"
        );
    }
}

impl<T> SubringPolynomial<T>
where
    T: Clone + Default + std::ops::Mul<Output = T> + std::ops::AddAssign + std::ops::SubAssign,
{
    /// Computes multiplication modulo `Y^k + 1`.
    ///
    /// Terms with degree at least `k` wrap with a sign change because
    /// `Y^k = -1` in the quotient ring.
    pub fn negacyclic_mul(&self, rhs: &Self) -> Self {
        self.assert_same_k(rhs);

        let k = self.k();
        let mut result = vec![T::default(); k];

        for (i, lhs_coeff) in self.coeffs.iter().enumerate() {
            for (j, rhs_coeff) in rhs.coeffs.iter().enumerate() {
                let product = lhs_coeff.clone() * rhs_coeff.clone();
                let degree = i + j;

                if degree < k {
                    result[degree] += product;
                } else {
                    result[degree - k] -= product;
                }
            }
        }

        Self::new(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubringDecomposition {
    d: usize,
    k: usize,
}

impl SubringDecomposition {
    pub fn new(d: usize, k: usize) -> Self {
        assert!(d > 0, "d must be positive");
        assert!(k > 0, "k must be positive");

        Self { d, k }
    }

    pub fn d(&self) -> usize {
        self.d
    }

    pub fn k(&self) -> usize {
        self.k
    }

    pub fn ring_degree(&self) -> usize {
        self.d * self.k
    }

    /// Implements Vec_k^d.
    ///
    /// Returns d vectors, each containing the coefficients whose
    /// exponents are congruent modulo d.
    pub fn decompose<T: Clone>(&self, coeffs: &[T]) -> Vec<SubringPolynomial<T>> {
        assert_eq!(
            coeffs.len(),
            self.ring_degree(),
            "coefficient vector length must equal d*k"
        );

        let mut parts = Vec::with_capacity(self.d);

        for i in 0..self.d {
            let mut coeffs_i = Vec::with_capacity(self.k);

            for j in 0..self.k {
                coeffs_i.push(coeffs[i + j * self.d].clone());
            }

            parts.push(SubringPolynomial::new(coeffs_i));
        }

        parts
    }

    /// Inverse of Vec_k^d.
    pub fn compose<T: Clone>(&self, parts: &[SubringPolynomial<T>]) -> Vec<T> {
        assert_eq!(parts.len(), self.d);

        for part in parts {
            assert_eq!(part.k(), self.k);
        }
        let mut coeffs = Vec::with_capacity(self.ring_degree());

        for j in 0..self.k {
            for part in parts {
                coeffs.push(part.coefficients()[j].clone());
            }
        }

        coeffs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subring_polynomial_reports_coefficients_and_k() {
        let polynomial = SubringPolynomial::new(vec![2_i64, 3, 5, 7]);

        assert_eq!(polynomial.k(), 4);
        assert_eq!(polynomial.coefficients(), &[2, 3, 5, 7]);
    }

    #[test]
    fn subring_polynomial_adds_and_subtracts_coefficient_wise() {
        let lhs = SubringPolynomial::new(vec![1_i64, 2, 3]);
        let rhs = SubringPolynomial::new(vec![4_i64, 5, 6]);

        assert_eq!(lhs.add(&rhs).coefficients(), &[5, 7, 9]);
        assert_eq!(lhs.sub(&rhs).coefficients(), &[-3, -3, -3]);
    }

    #[test]
    fn negacyclic_multiplication_wraps_with_sign_change() {
        // In Z[Y] / (Y^4 + 1):
        //
        // (1 + 2Y + 3Y^2 + 4Y^3)(5 + 6Y + 7Y^2 + 8Y^3)
        // = -56 - 36Y + 2Y^2 + 60Y^3.
        let lhs = SubringPolynomial::new(vec![1_i64, 2, 3, 4]);
        let rhs = SubringPolynomial::new(vec![5_i64, 6, 7, 8]);

        let product = lhs.negacyclic_mul(&rhs);

        assert_eq!(product.coefficients(), &[-56, -36, 2, 60]);
    }

    #[test]
    fn negacyclic_multiplication_respects_y_to_the_k_equals_minus_one() {
        // For k = 4, Y^3 * Y = Y^4 = -1.
        let y_cubed = SubringPolynomial::new(vec![0_i64, 0, 0, 1]);
        let y = SubringPolynomial::new(vec![0_i64, 1, 0, 0]);

        let product = y_cubed.negacyclic_mul(&y);

        assert_eq!(product.coefficients(), &[-1, 0, 0, 0]);
    }

    #[test]
    fn into_coefficients_returns_owned_storage() {
        let polynomial = SubringPolynomial::new(vec![3_i64, 1, 4]);

        assert_eq!(polynomial.into_coefficients(), vec![3, 1, 4]);
    }

    #[test]
    fn reports_dimensions() {
        let dec = SubringDecomposition::new(3, 4);

        assert_eq!(dec.d(), 3);
        assert_eq!(dec.k(), 4);
        assert_eq!(dec.ring_degree(), 12);
    }

    #[test]
    fn decomposition_layout_matches_paper() {
        let dec = SubringDecomposition::new(3, 4);

        let coeffs: Vec<u64> = (0..12).collect();

        let parts = dec.decompose(&coeffs);

        assert_eq!(parts[0].coefficients(), &[0, 3, 6, 9]);
        assert_eq!(parts[1].coefficients(), &[1, 4, 7, 10]);
        assert_eq!(parts[2].coefficients(), &[2, 5, 8, 11]);
    }

    #[test]
    fn compose_inverts_decompose() {
        let dec = SubringDecomposition::new(4, 3);

        let coeffs: Vec<i32> = (0..12).collect();

        let recovered = dec.compose(&dec.decompose(&coeffs));

        assert_eq!(recovered, coeffs);
    }
}
