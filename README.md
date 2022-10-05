# 🎤 vibrato: VIterbi-Based acceleRAted TOkenizer

[![Crates.io](https://img.shields.io/crates/v/vibrato)](https://crates.io/crates/vibrato)
[![Documentation](https://docs.rs/vibrato/badge.svg)](https://docs.rs/vibrato)
![Build Status](https://github.com/daac-tools/vibrato/actions/workflows/rust.yml/badge.svg)

Vibrato is a fast implementation of tokenization (or morphological analysis) based on the Viterbi algorithm.

The technical ideas behind this library appear in [this blog post](https://tech.legalforce.co.jp/entry/2022/09/20/133132) (Japanese).

## Features

### Fast tokenization

Vibrato is a Rust reimplementation of the fast tokenizer [MeCab](https://taku910.github.io/mecab/),
although its implementation has been simplified and optimized for even faster tokenization.
Especially for language resources with a large matrix
(e.g., [`unidic-cwj-3.1.1`](https://clrd.ninjal.ac.jp/unidic/back_number.html#unidic_cwj) with a matrix of 459 MiB),
Vibrato will run faster thanks to cache-efficient id mappings.

For example, the following figure shows an experimental result of
tokenization time with MeCab and its reimplementations.
The detailed experimental settings and other results are available on [Wiki](https://github.com/daac-tools/vibrato/wiki/Speed-Comparison).

![](./figures/comparison.svg)

### MeCab compatibility

Vibrato supports options for outputting tokenized results identical to MeCab, such as ignoring whitespace.

## Basic usage

This software is implemented in Rust.
First of all, install `rustc` and `cargo` following the [official instructions](https://www.rust-lang.org/tools/install).

### 1. Resource preparation

You can compile a system dictionary from language resources in the MeCab format.
The simplest way is using publicly-available resources such as IPADIC or UniDic.

The directory `scripts` provides scripts to prepare system dictionaries from several public resources.

```
$ ls -1 scripts
prepare_ipadic-mecab-2_7_0.sh
prepare_ipadic-mecab-neologd-20200910.sh
prepare_unidic-cwj-3_1_1.sh
prepare_unidic-mecab-2_1_2.sh
```

For example, if you want to use [mecab-ipadic v2.7.0](https://taku910.github.io/mecab/), run `prepare_ipadic-mecab-2_7_0.sh`.

```
$ ./scripts/prepare_ipadic-mecab-2_7_0.sh
```

The system dictionary `resources_ipadic-mecab-2_7_0/system.dic` will be produced.

```
$ ls resources_ipadic-mecab-2_7_0
system.dic
```

See the [document](./prepare/README.md) for preparation steps without these scripts.

### 2. Tokenization

To tokenize sentences using the system dictionary, run the following command.

```
$ echo '本とカレーの街神保町へようこそ。' | cargo run --release -p tokenize -- -i resources_ipadic-mecab-2_7_0/system.dic
```

The resultant tokens will be output in the Mecab format.

```
本	名詞,一般,*,*,*,*,本,ホン,ホン
と	助詞,並立助詞,*,*,*,*,と,ト,ト
カレー	名詞,固有名詞,地域,一般,*,*,カレー,カレー,カレー
の	助詞,連体化,*,*,*,*,の,ノ,ノ
街	名詞,一般,*,*,*,*,街,マチ,マチ
神保	名詞,固有名詞,地域,一般,*,*,神保,ジンボウ,ジンボー
町	名詞,接尾,地域,*,*,*,町,マチ,マチ
へ	助詞,格助詞,一般,*,*,*,へ,ヘ,エ
ようこそ	感動詞,*,*,*,*,*,ようこそ,ヨウコソ,ヨーコソ
。	記号,句点,*,*,*,*,。,。,。
EOS
```

If you want to output tokens separated by spaces, specify `-O wakati`.

```
$ echo '本とカレーの街神保町へようこそ。' | cargo run --release -p tokenize -- -i resources_ipadic-mecab-2_7_0/system.dic -O wakati
本 と カレー の 街 神保 町 へ ようこそ 。
```

### 3. Training

Vibrato also supports training a dictionary.
To train a dictionary, you must prepare at least the following six files.

* `corpus.txt`: Corpus file to be trained. The format is the same as the output of the `tokenize` command of Vibrato.
                The contents of the feature columns must match exactly with the columns of the lexicon file.
                If it differs even slightly, it is considered an unknown word.
* `train_lex.csv`: Lexicon file to be weighted. All connection IDs and weights must be set to 0.
* `train_unk.def`: Unknown word file to be weighted. All connection IDs and weights must be set to 0.
* `char.def`: Character definition file.
* `rewrite.def`: Rewrite rule definition file.
* `feature.def`: Feature definition file.

The file formats follow those in MeCab (see the [official document](https://taku910.github.io/mecab/learn.html)).
You can also find an example dataset [here](./vibrato/src/tests/resources).

Execute the following command to start the training process (Replace file names with the actual ones):
```
$ cargo run --release -p train -- \
    -t ./dataset/corpus.txt \
    -l ./dataset/train_lex.csv \
    -u ./dataset/train_unk.def \
    -c ./dataset/char.def \
    -f ./dataset/feature.def \
    -r ./dataset/rewrite.def \
    -o ./modeldata.zst
```

The training command supports multi-threading and changing some parameters.
See the `--help` message for more details.

When training is complete, the model is output to `./modeldata.zst`.

Next, run the following commands to generate a set of dictionary files from the model:

```
$ mkdir mydict # Prepare the output directory
$ cargo run --release -p dictgen -- \
    -i ./modeldata.zst \
    -l ./mydict/lex.csv \
    -u ./mydict/unk.def \
    -m ./mydict/matrix.def
```

Optionally, you can specify a user-defined dictionary to the `dictgen` command to automatically give connection IDs and weights.
See the `--help` message for more details.

After copying `dataset/char.def` under `mydict`, you can compile your system dictionary
following the [documentation](./prepare/README.md).

#### Accuracy evaluation

To split the input corpus randomly and output train/validation/test files, run the following command:

```
$ cargo run --release -p evaluate --bin split -- \
    -i ./dataset/corpus.txt \
    -t ./dataset/train.txt \
    -v ./dataset/valid.txt \
    -e ./dataset/test.txt
```

By default, 80% of the data is split into a training set, 10% into a validation set, and 10% into a test set.

To evaluate the accuracy, run the following command:

```
$ cargo run --release -p evaluate -- \
    -i ./system.dic \
    -t ./dataset/valid.txt \
    --feature-indices 0,1,2,3,9
```

where `--feature-indices` is an option to specify features' indices to determine correctness.
In this example, the 0th, 1st, 2nd, 3rd, and 9th features are considered.

## MeCab-compatible options

Vibrato is a reimplementation of the MeCab algorithm,
but with the default settings it can produce different tokens from MeCab.

For example, MeCab ignores spaces (more precisely, `SPACE` defined in `char.def`) in tokenization.

```
$ echo "mens second bag" | mecab
mens	名詞,固有名詞,組織,*,*,*,*
second	名詞,一般,*,*,*,*,*
bag	名詞,固有名詞,組織,*,*,*,*
EOS
```

However, Vibrato handles such spaces as tokens with the default settings.

```
$ echo 'mens second bag' | cargo run --release -p tokenize -- -i resources_ipadic-mecab-2_7_0/system.dic
mens	名詞,固有名詞,組織,*,*,*,*
 	記号,空白,*,*,*,*,*
second	名詞,固有名詞,組織,*,*,*,*
 	記号,空白,*,*,*,*,*
bag	名詞,固有名詞,組織,*,*,*,*
EOS
```

If you want to obtain the same results as MeCab, specify the arguments `-S` and `-M 24`.

```
$ echo 'mens second bag' | cargo run --release -p tokenize -- -i resources_ipadic-mecab-2_7_0/system.dic -S -M 24
mens	名詞,固有名詞,組織,*,*,*,*
second	名詞,一般,*,*,*,*,*
bag	名詞,固有名詞,組織,*,*,*,*
EOS
```

`-S` indicates if spaces are ignored.
`-M` indicates the maximum grouping length for unknown words.

### Notes

There are corner cases where tokenization results in different outcomes due to cost tiebreakers.
However, this would be not an essential problem.

## User dictionary

You can use your user dictionary along with the system dictionary.
The user dictionary must be in the CSV format.

```
<surface>,<left-id>,<right-id>,<cost>,<features...>
```

The first four columns are always required.
The others (i.e., `<features...>`) are optional.

For example,

```
$ cat user.csv
神保町,1293,1293,334,カスタム名詞,ジンボチョウ
本とカレーの街,1293,1293,0,カスタム名詞,ホントカレーノマチ
ようこそ,3,3,-1000,感動詞,ヨーコソ,Welcome,欢迎欢迎,Benvenuto,Willkommen
```

To use the user dictionary, specify the file with the `-u` argument.

```
$ echo '本とカレーの街神保町へようこそ。' | cargo run --release -p tokenize -- -i resources_ipadic-mecab-2_7_0/system.dic -u user.csv
本とカレーの街	カスタム名詞,ホントカレーノマチ
神保町	カスタム名詞,ジンボチョウ
へ	助詞,格助詞,一般,*,*,*,へ,ヘ,エ
ようこそ	感動詞,ヨーコソ,Welcome,欢迎欢迎,Benvenuto,Willkommen
。	記号,句点,*,*,*,*,。,。,。
EOS
```

# Smaller dictionary

When analyzing texts, Vibrato usually retrieves pre-computed bi-gram costs from `matrix.def`, a dictionary that stores connection costs similar to MeCab.
However, since the matrix is huge, Vibrato also supports the management of more compact dictionaries.

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

Next, compile the dictionary. Run the `prepare/system` command with the `--bigram-*` options instead of the `-m` option as follows:
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

The compact dictionary takes longer time than usual to analyze because it does not compute the bi-gram cost before hand.
Compiling the `tokenize` command with the `target-feature=+avx` option will reduce the analyzing time:
```
$ RUSTFLAGS='-C target-feature=+avx2' cargo build --release -p tokenize
```

## Benchmark

You can measure the tokenization speed for sentences in `test.txt`.

If you can guarantee that `system.dic` is exported from this library,
you can specify `--features=unchecked` for faster tokenization.

```
$ cargo run --release -p benchmark --features=unchecked -- -i resources_ipadic-mecab-2_7_0/system.dic < test.txt
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgment

The initial version of this software was developed by LegalForce, Inc.,
but not an officially supported LegalForce product.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
