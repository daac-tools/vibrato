use anyhow::{anyhow, Result};

use super::Connector;

impl Connector {
    // num_right, num_left
    // r0 l0
    // r0 l1
    // r0 l2
    // ...
    pub fn from_lines<I, L>(lines: I) -> Result<Self>
    where
        I: IntoIterator<Item = L>,
        L: AsRef<str>,
    {
        let mut lines = lines.into_iter();
        let (num_right, num_left) = Self::parse_header(lines.next().unwrap().as_ref())?;
        let mut data = vec![0; num_right * num_left];
        for line in lines {
            let line = line.as_ref();
            if !line.is_empty() {
                let (right_id, left_id, conn_cost) = Self::parse_body(line)?;
                data[left_id * num_right + right_id] = conn_cost;
            }
        }
        Ok(Self::new(data, num_right, num_left))
    }

    fn parse_header(line: &str) -> Result<(usize, usize)> {
        let cols: Vec<_> = line.split(' ').collect();
        if cols.len() != 2 {
            Err(anyhow!("Invalid format: {}", line))
        } else {
            Ok((cols[0].parse()?, cols[1].parse()?))
        }
    }

    fn parse_body(line: &str) -> Result<(usize, usize, i16)> {
        let cols: Vec<_> = line.split(' ').collect();
        if cols.len() != 3 {
            Err(anyhow!("Invalid format: {}", line))
        } else {
            Ok((cols[0].parse()?, cols[1].parse()?, cols[2].parse()?))
        }
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
        let conn = Connector::from_lines(data.split('\n')).unwrap();
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
        let conn = Connector::from_lines(data.split('\n')).unwrap();
        assert_eq!(conn.cost(0, 0), 0);
        assert_eq!(conn.cost(0, 1), 1);
        assert_eq!(conn.cost(0, 2), 2);
        assert_eq!(conn.cost(1, 0), -3);
        assert_eq!(conn.cost(1, 1), -4);
        assert_eq!(conn.cost(1, 2), -5);
    }
}
