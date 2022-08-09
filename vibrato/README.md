# vibrato

Vibrato is a fast implementation of tokenization (or morphological analysis) based on the viterbi algorithm.

## Examples

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;

use vibrato::{Dictionary, Tokenizer};

let file = File::open("src/tests/resources/system.dic").unwrap();
let dict = Dictionary::read(BufReader::new(file)).unwrap();

let mut tokenizer = vibrato::Tokenizer::new(&dict);
let tokens = tokenizer.tokenize("京都東京都").unwrap();

assert_eq!(tokens.len(), 2);

assert_eq!(tokens.get(0).surface().deref(), "京都");
assert_eq!(tokens.get(0).range_char(), 0..2);
assert_eq!(tokens.get(0).range_byte(), 0..6);

assert_eq!(tokens.get(1).surface().deref(), "東京都");
assert_eq!(tokens.get(1).range_char(), 2..5);
assert_eq!(tokens.get(1).range_byte(), 6..15);
```

## Feature flags

 - `unchecked`: Allows faster tokenization.
   Activate it only if you can guarantee that the input file is exported from this library

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
