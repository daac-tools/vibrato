use super::{LexType, Lexicon, RawWordEntry, WordFeats, WordMap, WordParam, WordParams};

impl Lexicon {
    pub fn from_lines<I, S>(lines: I, lex_type: LexType) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let entries: Vec<_> = lines
            .into_iter()
            .map(|l| Self::parse_csv(l.as_ref()))
            .collect();
        let map = WordMap::from_iter(entries.iter().map(|e| &e.surface));
        let params = WordParams::from_iter(entries.iter().map(|e| e.param));
        let feats = WordFeats::from_iter(entries.iter().map(|e| &e.feat));
        Self {
            map,
            params,
            feats,
            lex_type,
        }
    }

    fn parse_csv(line: &str) -> RawWordEntry {
        let items: Vec<_> = line.split(',').collect();
        assert!(4 <= items.len());
        let feat = if 4 < items.len() {
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
            feat,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let line = "東京,1,2,3";
        let entry = Lexicon::parse_csv(line);
        assert_eq!(
            entry,
            RawWordEntry {
                surface: "東京".to_string(),
                param: WordParam::new(1, 2, 3),
                feat: "".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_with_feat() {
        let line = "東京,1,2,3,京都,名詞,固有名詞";
        let entry = Lexicon::parse_csv(line);
        assert_eq!(
            entry,
            RawWordEntry {
                surface: "東京".to_string(),
                param: WordParam::new(1, 2, 3),
                feat: "京都,名詞,固有名詞".to_string(),
            }
        );
    }
}
