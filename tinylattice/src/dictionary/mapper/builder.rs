use std::io::{prelude::*, BufReader, Read};

use anyhow::{anyhow, Result};

use super::ConnIdMapper;

impl ConnIdMapper {
    pub fn from_reader<R>(l_rdr: R, r_rdr: R) -> Result<Self>
    where
        R: Read,
    {
        let left = Self::read(l_rdr)?;
        let right = Self::read(r_rdr)?;
        Ok(Self { left, right })
    }

    fn read<R>(rdr: R) -> Result<Vec<(u16, u16)>>
    where
        R: Read,
    {
        let reader = BufReader::new(rdr);
        let lines = reader.lines();

        let mut mapping = vec![(0, 0)];
        for line in lines {
            let line = line?;
            let cols: Vec<_> = line.split('\t').collect();
            if cols.is_empty() {
                return Err(anyhow!("Invalid format: {}", line));
            }
            let old_id = cols[0].parse()?;
            if old_id == 0 {
                return Err(anyhow!("Id zero is reserved: {}", line));
            }
            mapping.push((old_id, 0));
        }

        for new_id in 1..mapping.len() {
            let old_id = mapping[new_id].0 as usize;
            mapping[old_id].1 = new_id as u16;
        }

        Ok(mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let data = "2\n3\n4\n1\n";
        let mapping = ConnIdMapper::read(data.as_bytes()).unwrap();
        assert_eq!(mapping, vec![(0, 0), (2, 4), (3, 1), (4, 2), (1, 3),]);
    }
}
