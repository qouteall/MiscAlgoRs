use crate::functional::lazy_eval::Cache;

pub struct Matrix2D<T> {
    // it's row-major, each row is stored together
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T> Matrix2D<T> {
    fn index(&self, row: usize, col: usize) -> usize {
        assert!(row < self.rows, "row index out of bound");
        assert!(col < self.cols, "col index out of bound");
        row * self.cols + col
    }
    
    pub fn at(&self, row: usize, col: usize) -> &T {
        let index = self.index(row, col);
        &self.data[index]
    }
    
    pub fn at_mut(&mut self, row: usize, col: usize) -> &mut T {
        let index = self.index(row, col);
        &mut self.data[index]
    }
    
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        let index = self.index(row, col);
        self.data[index] = value;
    }
    
    pub fn borrow_row(&self, row: usize) -> &[T] {
        let start = self.index(row, 0);
        &self.data[start..(start + self.cols)]
    }
    
    pub fn borrow_row_mut(&mut self, row: usize) -> &mut [T] {
        let start = self.index(row, 0);
        &mut self.data[start..(start + self.cols)]
    }
    
    pub fn column_iter(&self, col: usize) -> impl Iterator<Item=&T> {
        (0..self.rows).map(move |row| self.at(row, col))
    }
}

impl<T: Clone> Matrix2D<T> {
    pub fn new(rows: usize, cols: usize, default: T) -> Self {
        Matrix2D {
            data: vec![default; rows * cols],
            rows,
            cols,
        }
    }
}

impl<T: Default> Matrix2D<T> {
    pub fn new_defaulted(rows: usize, cols: usize) -> Self {
        let mut vec = Vec::with_capacity(rows * cols);
        for _ in 0..rows * cols {
            vec.push(T::default());
        }
        
        Matrix2D { data: vec, rows, cols }
    }
}

impl<T: Clone> Cache<(usize, usize), T> for Matrix2D<Option<T>> {
    fn get_from_cache<>(&self, key: &(usize, usize)) -> Option<T> {
        self.at(key.0, key.1).clone()
    }
    
    fn put_to_cache(&mut self, key: &(usize, usize), value: T) {
        self.set(key.0, key.1, Some(value));
    }
}