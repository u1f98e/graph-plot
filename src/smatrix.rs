#[derive(Default)]
pub struct SMatrix {
    size: usize,
    data: Vec<Vec<f32>>,
}

#[derive(Debug)]
pub struct JacobiResult {
    eigenvalues: Vec<f32>,
    eigenvectors: Vec<Vec<f32>>,
}

impl SMatrix {
    pub fn from_vec(data: Vec<Vec<f32>>) -> Self {
        let size = data.len();
        Self { size, data }
    }

    pub fn identity(size: usize) -> Self {
        let mut data: Vec<Vec<f32>> = Vec::new();
        for i in 0..size {
            let mut row: Vec<f32> = Vec::new();
            for j in 0..size {
                row.push(if i == j { 1.0 } else { 0.0 });
            }
            data.push(row);
        }

        Self { size, data }
    }

    pub fn diagnonal(&self) -> Vec<f32> {
        let mut diagonal: Vec<f32> = Vec::new();
        for i in 0..self.data.len() {
            diagonal.push(self.data[i][i]);
        }
        diagonal
    }

    pub fn clone_region(&self, row: usize, col: usize, size: usize) -> Self {
        let mut data: Vec<Vec<f32>> = Vec::new();
        for row in self.data[row..(row + size)].into_iter() {
            data.push(row[col..(col + size)].into());
        }

        Self { size, data }
    }

    pub fn determinant(&self) -> f32 {
        let sign = |i, j| i32::pow(-1, i + j);
        let mut determinant = 0.0;
        for (row, row_data) in self.data.iter().enumerate() {
            determinant += sign(row as u32, 0) as f32
                * row_data[0]
                * self.clone_region(row, 1, self.size - 1).determinant();
        }

        determinant
    }

    pub fn jacobi2(&self) -> JacobiResult {
        // Index of largest off diagonal element in given row
        fn max_index(matrix: &SMatrix, row: usize) -> usize {
            let mut m = row + 1;
            for i in (row + 2)..matrix.size {
                if matrix.data[row][i].abs() > matrix.data[row][m].abs() {
                    m = i;
                }
            }
            return m
        }

        fn update(k: usize, t: f32, state: &mut usize, eigenvalues: &mut Vec<f32>, changed: &mut Vec<bool>) {
            eigenvalues[k] += t;
            if changed[k] && t.abs() < 1e-10 {
                changed[k] = false;
                *state -= 1;
            }
            else if !changed[k] && t.abs() > 1e-10 {
                changed[k] = true;
                *state += 1;
            }
        }

        fn rotate(matrix: &mut Vec<Vec<f32>>, c: f32, s: f32, (i, j, k, l): (usize, usize, usize, usize)) {
            let g = matrix[i][j];
            let h = matrix[k][l];
            matrix[i][j] = s * h + c * g;
            matrix[k][l] = c * h - s * g;
        }

        let mut matrix = self.data.clone();
        let mut eigenvectors = SMatrix::identity(self.size).data;
        let mut state = self.size;

        let mut indices = Vec::new();
        let mut eigenvalues = Vec::new();
        let mut changed = Vec::new();
        for i in 0..self.size {
            indices.push(max_index(&self, i));
            eigenvalues.push(self.data[i][i]);
            changed.push(true);
        }

        while state != 0 {
            // Find index (k,l) of pivot p
            let mut m = 0;
            for i in 1..(self.size - 1) {
                if matrix[i][indices[i]].abs() > matrix[m][indices[m]].abs() {
                    m = i;
                }
            }

            let k = m;
            let l = indices[m];
            let p = matrix[k][l];

            // Calculate c = cos(theta) and s = sin(theta)
            let y = (eigenvalues[0] - eigenvalues[k]) / 2.0;
            let d = y.abs() + (p.powi(2) + y.powi(2)).sqrt();
            let r = (p.powi(2) + d.powi(2)).sqrt();
            let c = d / r;
            let mut s = p / r;
            let mut t = p.powi(2) / d;

            if y < 0.0 {
                s = -s;
                t = -t;
            }

            matrix[k][l] = 0.0;
            update(k, -t, &mut state, &mut eigenvalues, &mut changed);
            update(l, t, &mut state, &mut eigenvalues, &mut changed);

            for i in 0..k {
                rotate(&mut matrix, c, s, (i, l, i, k));
            }
            for i in (k + 1)..l {
                rotate(&mut matrix, c, s, (i, l, k, i));
            }
            for i in (l + 1)..self.size {
                rotate(&mut matrix, c, s, (l, i, k, i));
            }

            // Rotate eigenvectors
            for i in 0..self.size {
                rotate(&mut eigenvectors, c, s, (i, l, i, k));
            }

            // Update indices
            for i in 0..self.size {
                indices[i] = max_index(&self, i);
            }
        }

        JacobiResult { eigenvalues, eigenvectors }
    }

    /// Returns eigenvalues and eigenvectors of a square matrix
    /// 
    /// Based on the c++ code here:
    /// https://people.sc.fsu.edu/~jburkardt/cpp_src/jacobi_eigenvalue/jacobi_eigenvalue.cpp
    /// 
    /// Why do mathmaticians insist on single letter variable names ugh
    pub fn jacobi(&self, max_iterations: u32) -> JacobiResult {
        /// Returns the Frobenius norm of a matrix
        fn norm_fro(matrix: &Vec<Vec<f32>>) -> f32 {
            let mut sum = 0.0;
            for row in matrix.iter() {
                for col in row.iter() {
                    sum += col.powi(2);
                }
            }
            sum.sqrt()
        }

        /// Determines the error in a right handed eigensystem
        fn eigen_error(
            matrix: &SMatrix,
            eigenvectors: Vec<Vec<f32>>,
            eigenvalues: Vec<f32>,
        ) -> f32 {
            let mut c = Vec::new();
            for row in 0..eigenvectors.len() {
                c.insert(row, Vec::new());
                for col in 0..matrix.size {
                    c[row].insert(col, 0.0);
                    for i in 0..matrix.size {
                        c[row][col] += matrix.data[i][col] * eigenvectors[row][i];
                    }

                    c[row][col] -= eigenvalues[row] * eigenvectors[row][col];
                }
            }

            norm_fro(&c)
        }

        let mut matrix = self.data.clone();
        let mut diagonals = self.diagnonal();
        let mut eigenvalues = diagonals.clone();
        let mut eigenvectors: Vec<Vec<f32>> = self.data.clone();
        let mut zw = vec![0.0; self.size];
        let mut rotations = 0;

        for row in eigenvectors.iter_mut() {
            for col in row.iter_mut() {
                *col = 0.0;
            }
        }

        for iteration in 0..max_iterations {
            let convergence_threshold = self.data
                .iter().flatten()
                .map(|x| x.powi(2))
                .sum::<f32>()
                .sqrt()
                / (4.0 * self.size as f32);

            if convergence_threshold == 0.0 {
                break;
            }

            for row in 0..self.size {
                for col in (row + 1)..self.size {
                    let gap = 10.0 * self.data[row][col].abs();
                    let value = matrix[row][col];

                    // Discard negligible off-diagonal elements
                    if iteration > 4 && gap == 0.0 {
                        matrix[row][col] = 0.0;
                    }
                    // Apply a rotation
                    else if convergence_threshold <= value {
                        let diff = eigenvalues[col] - eigenvalues[row];
                        let t = if gap == 0.0 {
                            value / diff
                        }
                        else {
                            let theta = 0.5 * diff / value;
                            (1.0 / (theta.abs() + (1.0 + theta.powi(2)).sqrt())) * theta.signum()
                        };

                        let c = 1.0 / (1.0 + t.powi(2)).sqrt();
                        let s = t * c;
                        let tau = s / (1.0 + c);
                        let correction = t * value;

                        // Accumulate corrections to diagonal elements
                        zw[row] -= correction;
                        zw[col] += correction;
                        eigenvalues[row] -= correction;
                        eigenvalues[col] += correction;

                        matrix[row][col] = 0.0;

                        // Rotate, using information from the upper triangle of A only 
                        for i in 0..row {
                            let g = matrix[row][i];
                            let h = matrix[col][i];
                            matrix[row][i] -= s * (h + g * tau);
                            matrix[col][i] += s * (g - h * tau);
                        }
                        for i in (row + 1)..col {
                            let g = matrix[i][row];
                            let h = matrix[col][i];
                            matrix[i][row] -= s * (h + g * tau);
                            matrix[col][i] += s * (g - h * tau);
                        }
                        for i in (col + 1)..self.size {
                            let g = matrix[i][row];
                            let h = matrix[i][col];
                            matrix[i][row] -= s * (h + g * tau);
                            matrix[i][col] += s * (g - h * tau);
                        }

                        // Accumulate information in the eigenvector matrix
                        for i in 0..self.size {
                            let g = eigenvectors[row][i];
                            let h = eigenvectors[col][i];
                            eigenvectors[row][i] -= s * (h + g * tau);
                            eigenvectors[col][i] += s * (g - h * tau);
                        }
                        rotations += 1;
                    }
                }
            }

            for i in 0..self.size {
                diagonals[i] += zw[i];
                eigenvalues[i] = diagonals[i];
                zw[i] = 0.0;
            }
        }

        // Restore upper triangle of input matrix
        for row in 0..self.size {
            for col in 0..row {
                matrix[row][col] = matrix[col][row];
            }
        }

        JacobiResult {
            eigenvalues,
            eigenvectors,
        }
    }
}
