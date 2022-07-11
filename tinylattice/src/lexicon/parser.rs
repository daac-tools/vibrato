use super::{RawWordEntry, WordParam};

pub fn entries_from_csv<I, S>(lines: I) -> Vec<RawWordEntry>
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

fn parse_csv(line: &str) -> RawWordEntry {
    let items: Vec<_> = line.split(',').collect();
    assert!(4 <= items.len());

    let info = if 4 < items.len() {
        items[4..].join(",")
    } else {
        String::new()
    };

    RawWordEntry {
        surface: items[0].to_string(),
        param: WordParam::new(
            items[1].parse().unwrap(),
            items[2].parse().unwrap(),
            items[3].parse().unwrap(),
        ),
        info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let data = "京都,0,1,2
東,3,4,5
東京,6,7,8";
        let entries = entries_from_csv(data.split('\n'));
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0],
            RawWordEntry {
                surface: "京都".to_string(),
                param: WordParam::new(0, 1, 2),
                info: String::new()
            }
        );
        assert_eq!(
            entries[1],
            RawWordEntry {
                surface: "東".to_string(),
                param: WordParam::new(3, 4, 5),
                info: String::new()
            }
        );
        assert_eq!(
            entries[2],
            RawWordEntry {
                surface: "東京".to_string(),
                param: WordParam::new(6, 7, 8),
                info: String::new()
            }
        );
    }

    #[test]
    fn test_parse_with_info() {
        let data = "京都,0,1,2,京都,名詞
東,3,4,5,東,名詞
東京,6,7,8,京都,名詞,固有名詞";
        let entries = entries_from_csv(data.split('\n'));
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0],
            RawWordEntry {
                surface: "京都".to_string(),
                param: WordParam::new(0, 1, 2),
                info: "京都,名詞".to_string(),
            }
        );
        assert_eq!(
            entries[1],
            RawWordEntry {
                surface: "東".to_string(),
                param: WordParam::new(3, 4, 5),
                info: "東,名詞".to_string(),
            }
        );
        assert_eq!(
            entries[2],
            RawWordEntry {
                surface: "東京".to_string(),
                param: WordParam::new(6, 7, 8),
                info: "京都,名詞,固有名詞".to_string(),
            }
        );
    }
}
