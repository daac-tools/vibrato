use super::CostMatrix;

pub fn matrix_from_text<I, L>(mut lines: I) -> CostMatrix
where
    I: Iterator<Item = L>,
    L: AsRef<str>,
{
    let (num_right, num_left) = parse_header(lines.next().unwrap().as_ref());
    let mut data = vec![0; num_right * num_left];
    for line in lines {
        let (left, right, cost) = parse_body(line.as_ref());
        data[right * num_left + left] = cost;
    }
    CostMatrix::new(data, num_left, num_right)
}

fn parse_header(line: &str) -> (usize, usize) {
    let items: Vec<_> = line.split(' ').collect();
    assert_eq!(items.len(), 2, "{:?}", &items);
    (items[0].parse().unwrap(), items[1].parse().unwrap())
}

fn parse_body(line: &str) -> (usize, usize, i16) {
    let items: Vec<_> = line.split(' ').collect();
    assert_eq!(items.len(), 3);
    (
        items[0].parse().unwrap(),
        items[1].parse().unwrap(),
        items[2].parse().unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_2x2() {
        let data = "2 2
0 0 0
0 1 1
1 0 2
1 1 3";
        let matrix = matrix_from_text(data.split('\n'));
        assert_eq!(matrix.cost(0, 0), 0);
        assert_eq!(matrix.cost(0, 1), 1);
        assert_eq!(matrix.cost(1, 0), 2);
        assert_eq!(matrix.cost(1, 1), 3);
    }
}
