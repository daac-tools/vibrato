use csv_core::ReadFieldResult;

pub trait FromU32 {
    fn from_u32(src: u32) -> Self;
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl FromU32 for usize {
    #[inline(always)]
    fn from_u32(src: u32) -> Self {
        // Since the pointer width is guaranteed to be 32 or 64,
        // the following process always succeeds.
        unsafe { Self::try_from(src).unwrap_unchecked() }
    }
}

pub fn parse_csv_row(row: &str) -> Vec<String> {
    let mut features = vec![];
    let mut rdr = csv_core::Reader::new();
    let mut bytes = row.as_bytes();
    let mut output = [0; 4096];
    loop {
        let (result, nin, nout) = rdr.read_field(bytes, &mut output);
        let end = match result {
            ReadFieldResult::InputEmpty => true,
            ReadFieldResult::Field { .. } => false,
            _ => unreachable!(),
        };
        features.push(std::str::from_utf8(&output[..nout]).unwrap().to_string());
        if end {
            break;
        }
        bytes = &bytes[nin..];
    }
    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_row() {
        assert_eq!(
            &["名詞", "トスカーナ"],
            parse_csv_row("名詞,トスカーナ").as_slice()
        );
    }

    #[test]
    fn test_parse_csv_row_with_quote() {
        assert_eq!(
            &["名詞", "1,2-ジクロロエタン"],
            parse_csv_row("名詞,\"1,2-ジクロロエタン\"").as_slice()
        );
    }
}
