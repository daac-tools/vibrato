use super::ConnectionMatrix;

pub fn matrix_from_text<I, L>(mut lines: I) -> ConnectionMatrix
where
    I: Iterator<Item = L>,
    L: AsRef<str>,
{
    let (num_right, num_left) = parse_header(lines.next().unwrap().as_ref());
    let mut data = vec![0; num_right * num_left];
    for line in lines {
        let (i, j, cost) = parse_body(line.as_ref());
        data[i * num_left + j] = cost;
    }
    ConnectionMatrix::new(data, num_left, num_right)
}

fn parse_header(line: &str) -> (usize, usize) {
    let items: Vec<_> = line.split(',').collect();
    assert_eq!(items.len(), 2);
    (items[0].parse().unwrap(), items[1].parse().unwrap())
}

fn parse_body(line: &str) -> (usize, usize, i16) {
    let items: Vec<_> = line.split(',').collect();
    assert_eq!(items.len(), 3);
    (
        items[0].parse().unwrap(),
        items[1].parse().unwrap(),
        items[2].parse().unwrap(),
    )
}
