use ndarray::prelude::*;
use ndarray::ArrayBase;

pub fn extract_anti_diagonal_rect2<T>(arr: &ArrayBase<T, Ix2>) -> Vec<f64>
where
    T: ndarray::Data<Elem = f64>,
{
    let (nrows, ncols) = arr.dim();
    // Create empty Vec<f64>
    let mut numbers: Vec<f64> = Vec::new();
    if nrows < ncols {
        for i in 0..nrows {
            numbers.push(arr[[nrows - i - 1, i]]);
        }
    } else {
        for j in 0..nrows {
            if j < ncols {
                numbers.push(arr[[nrows - j - 1, j]]);
            }
        }
    }

    numbers
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_extract_anti_diagonal_rect2_square_matrix() {
        let matrix = array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];

        let result = extract_anti_diagonal_rect2(&matrix);
        let expected = vec![7.0, 5.0, 3.0]; // [2,0], [1,1] mais selon votre logique actuelle

        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_anti_diagonal_rect2_rectangular() {
        let matrix = array![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];

        let result = extract_anti_diagonal_rect2(&matrix);
        // Pour une 3x2, ça devrait prendre [2,0] et [1,1]
        assert_eq!(result.len(), 2);
    }
}
