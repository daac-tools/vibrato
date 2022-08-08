# 🎤 vibrato: VIterbi-Based acceleRAted TOkenizer

Vibrato is a fast implementation of tokenization (or morphological analysis) based on the viterbi algorithm.

## Features

 - Fast tokenization
 - MeCab compatible

## Example usage

This software is implemented in Rust.
First of all, install `rustc` and `cargo` following the [official instructions](https://www.rust-lang.org/tools/install).

### 1. Resource preparation

You can compile a system dictionary from language resources in the MeCab format.
The simplest way is using publicly-available resources such as IPADIC or UniDic.

The directory `scripts` provides scripts to prepare system dictionaries from several public resources.

```
$ ls -1 scripts
prepare_ipadic-mecab-2_7_0.sh
prepare_unidic-cwj-3_1_0.sh
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
$ cat data/user.csv
神保町,1293,1293,334,カスタム名詞,ジンボチョウ
本とカレーの街,1293,1293,0,カスタム名詞,ホントカレーノマチ
ようこそ,3,3,-1000,感動詞,ヨーコソ,Welcome,欢迎欢迎,Benvenuto,Willkommen
```

To use the user dictionary, specify the file with the `-u` argument.

```
$ echo '本とカレーの街神保町へようこそ。' | cargo run --release -p tokenize -- -i resources_ipadic-mecab-2_7_0/system.dic -u data/user.csv
本とカレーの街	カスタム名詞,ホントカレーノマチ
神保町	カスタム名詞,ジンボチョウ
へ	助詞,格助詞,一般,*,*,*,へ,ヘ,エ
ようこそ	感動詞,ヨーコソ,Welcome,欢迎欢迎,Benvenuto,Willkommen
。	記号,句点,*,*,*,*,。,。,。
EOS
```

## Benchmark

You can measure the tokenization speed.

```
$ cargo run --release -p benchmark -- -i resources_ipadic-mecab-2_7_0/system.dic < data/wagahaiwa_nekodearu.txt
```

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
