#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>, // column-major: row + col * rows
}

impl<T: Clone + Default> Matrix<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![T::default(); rows * cols],
        }
    }
}

impl<T> Matrix<T> {
    pub fn from_vec_column_major(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(data.len(), rows * cols, "matrix data length mismatch");
        Self { rows, cols, data }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> &T {
        &self.data[self.index(row, col)]
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) {
        let idx = self.index(row, col);
        self.data[idx] = value;
    }

    pub fn raw(&self) -> &[T] {
        &self.data
    }

    pub fn raw_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn index(&self, row: usize, col: usize) -> usize {
        assert!(row < self.rows, "row index out of bounds");
        assert!(col < self.cols, "column index out of bounds");
        row + col * self.rows
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector<T> {
    data: Vec<T>,
}

impl<T: Clone + Default> Vector<T> {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![T::default(); size],
        }
    }
}

impl<T> Vector<T> {
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, i: usize) -> &T {
        &self.data[i]
    }

    pub fn set(&mut self, i: usize, value: T) {
        self.data[i] = value;
    }

    pub fn raw(&self) -> &[T] {
        &self.data
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchMatrix<T> {
    rows: usize,
    cols: usize,
    batches: usize,
    data: Vec<T>, // batch * rows * cols + row + col * rows
}

impl<T: Clone + Default> BatchMatrix<T> {
    pub fn new(rows: usize, cols: usize, batches: usize) -> Self {
        Self {
            rows,
            cols,
            batches,
            data: vec![T::default(); rows * cols * batches],
        }
    }
}

impl<T> BatchMatrix<T> {
    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn batches(&self) -> usize {
        self.batches
    }

    pub fn get(&self, batch: usize, row: usize, col: usize) -> &T {
        &self.data[self.index(batch, row, col)]
    }

    pub fn set(&mut self, batch: usize, row: usize, col: usize, value: T) {
        let idx = self.index(batch, row, col);
        self.data[idx] = value;
    }

    pub fn raw(&self) -> &[T] {
        &self.data
    }

    fn index(&self, batch: usize, row: usize, col: usize) -> usize {
        assert!(batch < self.batches, "batch index out of bounds");
        assert!(row < self.rows, "row index out of bounds");
        assert!(col < self.cols, "column index out of bounds");
        batch * self.rows * self.cols + row + col * self.rows
    }
}

pub type RealMatrix = Matrix<f64>;
pub type RealVector = Vector<f64>;
pub type RealBatchMatrix = BatchMatrix<f64>;

pub fn naive_matrix_mult(a: &RealMatrix, b: &RealMatrix) -> RealMatrix {
    assert_eq!(a.cols(), b.rows(), "matrix dimensions do not match");

    let mut out = RealMatrix::new(a.rows(), b.cols());

    for i in 0..a.rows() {
        for k in 0..a.cols() {
            for j in 0..b.cols() {
                let value = *out.get(i, j) + *a.get(i, k) * *b.get(k, j);
                out.set(i, j, value);
            }
        }
    }

    out
}

pub fn naive_apply_matrix(a: &RealMatrix, x: &RealVector) -> RealVector {
    assert_eq!(a.cols(), x.size(), "matrix/vector dimensions do not match");

    let mut out = RealVector::new(a.rows());

    for j in 0..a.cols() {
        for i in 0..a.rows() {
            let value = *out.get(i) + *a.get(i, j) * *x.get(j);
            out.set(i, value);
        }
    }

    out
}

pub fn naive_transpose<T: Clone + Default>(a: &Matrix<T>) -> Matrix<T> {
    let mut out = Matrix::new(a.cols(), a.rows());

    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(j, i, a.get(i, j).clone());
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_is_column_major() {
        let mut m = RealMatrix::new(2, 3);
        m.set(0, 0, 1.0);
        m.set(1, 0, 2.0);
        m.set(0, 1, 3.0);
        m.set(1, 1, 4.0);
        m.set(0, 2, 5.0);
        m.set(1, 2, 6.0);

        assert_eq!(m.raw(), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn naive_matrix_multiplication_works() {
        let mut a = RealMatrix::new(2, 2);
        a.set(0, 0, 1.0);
        a.set(1, 0, 2.0);
        a.set(0, 1, 3.0);
        a.set(1, 1, 4.0);

        let mut b = RealMatrix::new(2, 2);
        b.set(0, 0, 5.0);
        b.set(1, 0, 6.0);
        b.set(0, 1, 7.0);
        b.set(1, 1, 8.0);

        let c = naive_matrix_mult(&a, &b);

        assert_eq!(*c.get(0, 0), 23.0);
        assert_eq!(*c.get(1, 0), 34.0);
        assert_eq!(*c.get(0, 1), 31.0);
        assert_eq!(*c.get(1, 1), 46.0);
    }

    #[test]
    fn transpose_works() {
        let mut a = RealMatrix::new(2, 3);
        a.set(0, 0, 1.0);
        a.set(1, 0, 2.0);
        a.set(0, 1, 3.0);
        a.set(1, 1, 4.0);
        a.set(0, 2, 5.0);
        a.set(1, 2, 6.0);

        let t = naive_transpose(&a);

        assert_eq!(t.rows(), 3);
        assert_eq!(t.cols(), 2);
        assert_eq!(*t.get(0, 0), 1.0);
        assert_eq!(*t.get(1, 0), 3.0);
        assert_eq!(*t.get(2, 0), 5.0);
        assert_eq!(*t.get(0, 1), 2.0);
        assert_eq!(*t.get(1, 1), 4.0);
        assert_eq!(*t.get(2, 1), 6.0);
    }

    #[test]
    fn batch_matrix_is_column_major_per_batch() {
        let mut b = RealBatchMatrix::new(2, 2, 2);
        b.set(0, 0, 0, 1.0);
        b.set(0, 1, 0, 2.0);
        b.set(0, 0, 1, 3.0);
        b.set(0, 1, 1, 4.0);
        b.set(1, 0, 0, 5.0);

        assert_eq!(*b.get(0, 1, 1), 4.0);
        assert_eq!(*b.get(1, 0, 0), 5.0);
    }

    #[test]
    fn generic_matrix_supports_u64() {
        let mut m = Matrix::<u64>::new(2, 2);
        m.set(1, 1, 42);
        assert_eq!(*m.get(1, 1), 42);
    }
}
