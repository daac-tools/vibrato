# Compiling system dictionaries

This document describes how to compile system dictionaries,
assuming you are in the root directory of this repository.

You can compile system dictionaries from language resources in the [MeCab format](https://taku910.github.io/mecab/).
The simplest way is using publicly-available resources such as UniDic or IPADIC.

Here, consider to use `unidic-mecab-2.1.2`.

```shell
$ wget "https://clrd.ninjal.ac.jp/unidic_archive/cwj/2.1.2/unidic-mecab-2.1.2_src.zip" -O "./unidic-mecab-2.1.2_src.zip" --no-check-certificate
$ unzip unidic-mecab-2.1.2_src.zip
```

To compile the system dictionary from the resource,
run the following command.

```shell
$ cargo run --release -p compile -- \
    -l unidic-mecab-2.1.2_src/lex.csv \
    -m unidic-mecab-2.1.2_src/matrix.def \
    -u unidic-mecab-2.1.2_src/unk.def \
    -c unidic-mecab-2.1.2_src/char.def \
    -o system.dic.zst
```

Instead of using publicly-available trained resources,
you can manually train parameters from your own corpus
in the manner described in [train.md](./train.md).

## Accelerating your dictionaries

Vibrato supports editing your dictionary to achieve faster tokenization.
See [map.md](./map.md).

