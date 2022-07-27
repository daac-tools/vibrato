# 🎶 vibrato: VIterbi-Based acceleRAted TOkenizer

## Resource preparation

```
$ ./scripts/prepare_ipadic-mecab-2_7_0.sh
$ ./scripts/prepare_unidic-cwj-3_1_0.sh
$ ./scripts/prepare_unidic-mecab-2_1_2.sh
```

## Compiling dictionary

```
$ cargo run --release -p compile -- -r resources_ipadic-mecab-2_7_0 -o system.dic
Compiling the system dictionary...
0.9053542 seconds
Writting the system dictionary...
44.63689613342285 MiB
```

## Tokenize

```
$ echo '本とカレーの街神保町へようこそ。' | cargo run --release -p tokenize -- -i system.dic
Loading the dictionary...
Ready to tokenize :)
本      名詞,一般,*,*,*,*,本,ホン,ホン
と      助詞,並立助詞,*,*,*,*,と,ト,ト
カレー  名詞,固有名詞,地域,一般,*,*,カレー,カレー,カレー
の      助詞,連体化,*,*,*,*,の,ノ,ノ
街      名詞,一般,*,*,*,*,街,マチ,マチ
神保    名詞,固有名詞,地域,一般,*,*,神保,ジンボウ,ジンボー
町      名詞,接尾,地域,*,*,*,町,マチ,マチ
へ      助詞,格助詞,一般,*,*,*,へ,ヘ,エ
ようこそ        感動詞,*,*,*,*,*,ようこそ,ヨウコソ,ヨーコソ
。      記号,句点,*,*,*,*,。,。,。
EOS
```

```
$ echo '本とカレーの街神保町へようこそ。' | cargo run --release -p tokenize -- -i system.dic -m wakati
Loading the dictionary...
Ready to tokenize :)
本 と カレー の 街 神保 町 へ ようこそ 。
```

```
$ cat user.csv
本とカレー,カスタム名詞,ホントカレー
神保町,カスタム名詞,ジンボチョウ
$ echo '本とカレーの街神保町へようこそ。' | cargo run --release -p tokenize -- -i system.dic -u user.csv
Loading the dictionary...
Ready to tokenize :)
本とカレー      カスタム名詞,ホントカレー
の      助詞,連体化,*,*,*,*,の,ノ,ノ
街      名詞,一般,*,*,*,*,街,マチ,マチ
神保町  カスタム名詞,ジンボチョウ
へ      助詞,格助詞,一般,*,*,*,へ,ヘ,エ
ようこそ        感動詞,*,*,*,*,*,ようこそ,ヨウコソ,ヨーコソ
。      記号,句点,*,*,*,*,。,。,。
EOS
```

## Benchmark

```
$ cargo run --release -p benchmark -- -i system.dic < benchmark/data/wagahaiwa_nekodearu.txt
[benchmark/src/main.rs:50] n_words = 2462700
Warmup: 0.0813649
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
Number_of_sentences: 2376
Elapsed_seconds_to_tokenize_all_sentences: [0.07661468000000002,0.07816473125,0.08134009]
```
