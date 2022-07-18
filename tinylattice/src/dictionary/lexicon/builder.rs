use anyhow::{anyhow, Result};

use super::{LexType, Lexicon, RawWordEntry, WordFeatures, WordMap, WordParam, WordParams};

impl Lexicon {
    pub fn from_lines<I, S>(lines: I, lex_type: LexType) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut entries = vec![];
        for line in lines {
            entries.push(Self::parse_csv(line.as_ref())?);
        }
        let map = WordMap::from_iter(entries.iter().map(|e| &e.surface));
        let params = WordParams::from_iter(entries.iter().map(|e| e.param));
        let features = WordFeatures::from_iter(entries.iter().map(|e| &e.feature));
        Ok(Self {
            map,
            params,
            features,
            lex_type,
        })
    }

    fn parse_csv(line: &str) -> Result<RawWordEntry> {
        let cols: Vec<_> = line.split(',').collect();
        if cols.len() < 4 {
            return Err(anyhow!("Invalid format: {}", line));
        }
        let surface = cols[0].parse()?;
        let left_id = cols[1].parse()?;
        let right_id = cols[2].parse()?;
        let word_cost = cols[3].parse()?;
        let feature = cols.get(4..).map_or("".to_string(), |x| x.join(","));
        Ok(RawWordEntry {
            surface,
            param: WordParam::new(left_id, right_id, word_cost),
            feature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let line = "東京,1,2,3";
        let entry = Lexicon::parse_csv(line).unwrap();
        assert_eq!(
            entry,
            RawWordEntry {
                surface: "東京".to_string(),
                param: WordParam::new(1, 2, 3),
                feature: "".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_with_feature() {
        let line = "東京,1,2,3,京都,名詞,固有名詞";
        let entry = Lexicon::parse_csv(line).unwrap();
        assert_eq!(
            entry,
            RawWordEntry {
                surface: "東京".to_string(),
                param: WordParam::new(1, 2, 3),
                feature: "京都,名詞,固有名詞".to_string(),
            }
        );
    }
}
