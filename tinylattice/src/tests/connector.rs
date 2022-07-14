use crate::dictionary::connector::*;

const MATRIX_TEXT: &str = include_str!("./resources/matrix_10x10.def");

#[test]
fn test_matrix() {
    let conn = Connector::from_lines(MATRIX_TEXT.split('\n')).unwrap();
    assert_eq!(conn.num_left(), 10);
    assert_eq!(conn.num_right(), 10);
    assert_eq!(conn.cost(0, 0), 0);
    assert_eq!(conn.cost(0, 1), 863);
    assert_eq!(conn.cost(1, 0), -3689);
    assert_eq!(conn.cost(9, 9), -2490);
}
