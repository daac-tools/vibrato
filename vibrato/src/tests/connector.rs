use crate::dictionary::connector::*;

const MATRIX_DEF: &str = include_str!("./resources/matrix.def");

#[test]
fn test_matrix() {
    let conn = MatrixConnector::from_reader(MATRIX_DEF.as_bytes()).unwrap();
    assert_eq!(conn.num_left(), 10);
    assert_eq!(conn.num_right(), 10);
    assert_eq!(conn.cost(0, 0), 0);
    assert_eq!(conn.cost(0, 1), 863);
    assert_eq!(conn.cost(1, 0), -3689);
    assert_eq!(conn.cost(9, 9), -2490);
}
