use std::io::{prelude::*, BufReader, Read};

use anyhow::{anyhow, Result};

use super::ConnIdMapper;

impl ConnIdMapper {
    pub fn from_ranks<L, R>(l_ranks: L, r_ranks: R) -> Result<Self>
    where
        L: IntoIterator<Item = u16>,
        R: IntoIterator<Item = u16>,
    {
        let left = Self::compile(l_ranks)?;
        let right = Self::compile(r_ranks)?;
        Ok(Self { left, right })
    }

    fn compile<I>(ranks: I) -> Result<Vec<u16>>
    where
        I: IntoIterator<Item = u16>,
    {
        let mut old_ids = vec![0];
        for old_id in ranks {
            if old_id == 0 {
                return Err(anyhow!("Id zero is reserved"));
            }
            old_ids.push(old_id);
        }
        let mut new_ids = vec![0; old_ids.len()];
        for new_id in 1..old_ids.len() {
            let old_id = old_ids[new_id] as usize;
            assert_ne!(old_id, 0);
            new_ids[old_id] = new_id as u16;
        }
        Ok(new_ids)
    }

    pub fn from_reader<L, R>(l_rdr: L, r_rdr: R) -> Result<Self>
    where
        L: Read,
        R: Read,
    {
        let left = Self::read(l_rdr)?;
        let right = Self::read(r_rdr)?;
        Ok(Self { left, right })
    }

    fn read<R>(rdr: R) -> Result<Vec<u16>>
    where
        R: Read,
    {
        let reader = BufReader::new(rdr);
        let lines = reader.lines();

        let mut old_ids = vec![0];
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
            old_ids.push(old_id);
        }
        let mut new_ids = vec![0; old_ids.len()];
        for new_id in 1..old_ids.len() {
            let old_id = old_ids[new_id] as usize;
            assert_ne!(old_id, 0);
            new_ids[old_id] = new_id as u16;
        }
        Ok(new_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let data = "2\n3\n4\n1\n";
        let mapping = ConnIdMapper::read(data.as_bytes()).unwrap();
        assert_eq!(mapping, vec![0, 4, 1, 2, 3]);
    }
}
