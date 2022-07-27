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
$ echo 'ヴェネツィアはイタリアにあります。' | cargo run --release -p tokenize -- -i system.dic
Loading the dictionary...
Ready to tokenize :)
ヴェネツィア    名詞,一般,*,*,*,*,* (unk)
は      助詞,係助詞,*,*,*,*,は,ハ,ワ
イタリア        名詞,固有名詞,地域,国,*,*,イタリア,イタリア,イタリア
に      助詞,格助詞,一般,*,*,*,に,ニ,ニ
あり    動詞,自立,*,*,五段・ラ行,連用形,ある,アリ,アリ
ます    助動詞,*,*,*,特殊・マス,基本形,ます,マス,マス
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
