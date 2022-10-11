# Benchmarking

You can measure the tokenization speed using a system dictionary `system.dic`
and sentences in `test.txt` with the following command.

```
$ cargo run --release -p benchmark -- -i system.dic < test.txt
```

If you can guarantee that `system.dic` is exported from this library,
you can specify `--features=unchecked` for faster tokenization.

```
$ cargo run --release -p benchmark --features=unchecked -- -i system.dic < test.txt
```
