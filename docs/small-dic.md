# Generating smaller dictionaries

Vibrato provides an option to generate smaller a dictionary that stores connection costs in compressed space,
while sacrificing tokenization speed.
This documentation describes the generation steps. 

The following description assumes you are at the root directory of this repository.

## 1. Training

To generate a smaller dictionary, you need to prepare a trained model file from your corpus following the step 1 in [this document](./train.md).

## 2. Generating dictionary files

To generate a compact dictionary, give `--conn-id-info-out` option to the `dictgen` command as follows:
```
$ cargo run --release -p dictgen -- \
    -i ./modeldata.zst \
    -l ./mydict/lex.csv \
    -u ./mydict/unk.def \
    -m ./mydict/matrix.def \
    --conn-id-info-out ./mydict/bigram
```

This command generates three files: `./mydict/bigram.left`, `./mydict/bigram.right`, and `./mydict/bigram.cost`.

## 3. Compiling system dictionary

Run the `prepare/system` command with the `--bigram-*` options instead of the `-m` option as follows:
```
$ cargo run --release -p prepare --bin system -- \
    -l ./mydict/lex.csv \
    -u ./mydict/unk.def \
    -c ./mydict/char.def \
    --bigram-left-in ./mydict/bigram.left \
    --bigram-right-in ./mydict/bigram.right \
    --bigram-cost-in ./mydict/bigram.cost \
    -o system-compact.dic
```

The compiled dictionary `system-compact.dic` can be used in place of the system dictionary `system.dic` described above.

## SIMD acceleration

Compiling the `tokenize` command with the `target-feature=+avx2` option enables a SIMD acceleration (if your machine supports it) and will reduce the analyzing time:
```
$ RUSTFLAGS='-C target-feature=+avx2' cargo build --release -p tokenize
```
