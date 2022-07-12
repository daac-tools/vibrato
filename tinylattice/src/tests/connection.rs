use crate::dictionary::connection::*;

const MATRIX_TEXT: &str = include_str!("./resources/matrix_10x10.def");

#[test]
fn test_matrix() {
    let matrix = parser::matrix_from_text(MATRIX_TEXT.split('\n'));
    assert_eq!(matrix.num_left(), 10);
    assert_eq!(matrix.num_right(), 10);
    assert_eq!(matrix.cost(0, 0), 0);
    assert_eq!(matrix.cost(0, 1), 863);
    assert_eq!(matrix.cost(1, 0), -3689);
    assert_eq!(matrix.cost(9, 9), -2490);
}
