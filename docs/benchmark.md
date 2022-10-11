# Benchmarking

You can measure the tokenization speed for sentences in `test.txt`.

If you can guarantee that `system.dic` is exported from this library,
you can specify `--features=unchecked` for faster tokenization.

```
$ cargo run --release -p benchmark --features=unchecked -- -i resources_ipadic-mecab-2_7_0/system.dic < test.txt
```
