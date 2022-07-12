use super::Connector;

impl Connector {
    // num_right, num_left
    // r0 l0
    // r0 l1
    // r0 l2
    // ...
    pub fn from_lines<I, L>(mut lines: I) -> Self
    where
        I: Iterator<Item = L>,
        L: AsRef<str>,
    {
        let (num_right, num_left) = Self::parse_header(lines.next().unwrap().as_ref());
        let mut data = vec![0; num_right * num_left];
        for line in lines {
            let line = line.as_ref();
            if !line.is_empty() {
                let (right_id, left_id, cost) = Self::parse_body(line);
                data[left_id * num_right + right_id] = cost;
            }
        }
        Self::new(data, num_right, num_left)
    }

    fn parse_header(line: &str) -> (usize, usize) {
        let items: Vec<_> = line.split(' ').collect();
        assert_eq!(items.len(), 2, "{:?}", &items);
        (items[0].parse().unwrap(), items[1].parse().unwrap())
    }

    fn parse_body(line: &str) -> (usize, usize, i16) {
        let items: Vec<_> = line.split(' ').collect();
        assert_eq!(items.len(), 3, "{:?}", &items);
        (
            items[0].parse().unwrap(),
            items[1].parse().unwrap(),
            items[2].parse().unwrap(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2x2() {
        let data = "2 2
0 0 0
0 1 1
1 0 -2
1 1 -3";
        let conn = Connector::from_lines(data.split('\n'));
        assert_eq!(conn.cost(0, 0), 0);
        assert_eq!(conn.cost(0, 1), 1);
        assert_eq!(conn.cost(1, 0), -2);
        assert_eq!(conn.cost(1, 1), -3);
    }

    #[test]
    fn test_2x3() {
        let data = "2 3
0 0 0
0 1 1
0 2 2
1 0 -3
1 1 -4
1 2 -5";
        let conn = Connector::from_lines(data.split('\n'));
        assert_eq!(conn.cost(0, 0), 0);
        assert_eq!(conn.cost(0, 1), 1);
        assert_eq!(conn.cost(0, 2), 2);
        assert_eq!(conn.cost(1, 0), -3);
        assert_eq!(conn.cost(1, 1), -4);
        assert_eq!(conn.cost(1, 2), -5);
    }
}
