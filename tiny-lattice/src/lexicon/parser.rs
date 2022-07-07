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
