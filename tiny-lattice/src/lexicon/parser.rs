#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawLexiconEntry {
    pub surface: String,
    pub left_id: i16,
    pub right_id: i16,
    pub cost: i16,
}

pub fn entries_from_csv<I, S>(lines: I) -> Vec<RawLexiconEntry>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut entries = vec![];
    for line in lines {
        entries.push(parse_csv(line.as_ref()));
    }
    entries
}

fn parse_csv(line: &str) -> RawLexiconEntry {
    let items: Vec<_> = line.split(',').collect();
    RawLexiconEntry {
        surface: items[0].to_string(),
        left_id: items[1].parse().unwrap(),
        right_id: items[2].parse().unwrap(),
        cost: items[3].parse().unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_small() {
        let data = "京都,0,1,2
東,3,4,5
東京,6,7,8";
        let entries = entries_from_csv(data.split('\n'));
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0],
            RawLexiconEntry {
                surface: "京都".to_string(),
                left_id: 0,
                right_id: 1,
                cost: 2
            }
        );
        assert_eq!(
            entries[1],
            RawLexiconEntry {
                surface: "東".to_string(),
                left_id: 3,
                right_id: 4,
                cost: 5
            }
        );
        assert_eq!(
            entries[2],
            RawLexiconEntry {
                surface: "東京".to_string(),
                left_id: 6,
                right_id: 7,
                cost: 8
            }
        );
    }
}
