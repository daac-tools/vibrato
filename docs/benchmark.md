# Benchmarking

You can measure the tokenization speed using a system dictionary `system.dic.zst`
and sentences in `test.txt` with the following command.

```shell
$ cargo run --release -p benchmark -- -i system.dic.zst < test.txt
```

If you can guarantee that `system.dic.zst` is exported from this library,
you can specify `--features=unchecked` for faster tokenization.

```shell
$ cargo run --release -p benchmark --features=unchecked -- -i system.dic.zst < test.txt
```
