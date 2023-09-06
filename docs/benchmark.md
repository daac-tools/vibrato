# Benchmarking

You can measure the tokenization speed using a system dictionary `system.dic.zst`
and sentences in `test.txt` with the following command.

```
$ cargo run --release -p benchmark -- -i system.dic.zst < test.txt
```
