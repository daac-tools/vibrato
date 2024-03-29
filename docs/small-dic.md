# Generating smaller dictionaries

Vibrato provides an option to generate a smaller dictionary that stores connection costs in compressed space
while sacrificing tokenization speed.
This document describes the generation steps, assuming you are in the root directory of this repository.

See [this paper](https://www.anlp.jp/proceedings/annual_meeting/2023/pdf_dir/C2-1.pdf) for the technical details.

## 1. Preparing trained model

To generate a smaller dictionary, you need to prepare a trained model file following [Step 1 in this document](./train.md#1-training).

## 2. Generating dictionary files

To generate a smaller dictionary, specify the `--conn-id-info-out` option to the `dictgen` command as follows:

```
$ mkdir mydict # Prepare the output directory
$ cargo run --release -p dictgen -- \
    -i ./modeldata.zst \
    -l ./mydict/lex.csv \
    -u ./mydict/unk.def \
    -m ./mydict/matrix.def \
    --conn-id-info-out ./mydict/bigram
```

This command generates three files: `./mydict/bigram.left`, `./mydict/bigram.right`, and `./mydict/bigram.cost`.

## 3. Compiling system dictionary

After copying `char.def` used on the training to `mydict`,
run the `compile` command with the `--bigram-*` options instead of the `-m` option as follows:

```
$ cargo run --release -p compile -- \
    -l ./mydict/lex.csv \
    -u ./mydict/unk.def \
    -c ./mydict/char.def \
    --bigram-left-in ./mydict/bigram.left \
    --bigram-right-in ./mydict/bigram.right \
    --bigram-cost-in ./mydict/bigram.cost \
    --dual-connector # Optional argument for faster but larger model
    -o system-compact.dic.zst
```

The compiled dictionary `system-compact.dic.zst` can be used in place of
the system dictionary generated by specifying `-m mydict/matrix.def`
as described in [this document](./train.md).

## SIMD acceleration

Compiling the `tokenize` command with the `target-feature=+avx2` option enables a SIMD acceleration (if your machine supports it) and will reduce the analyzing time:

```
$ RUSTFLAGS='-C target-feature=+avx2' cargo build --release -p tokenize
```
