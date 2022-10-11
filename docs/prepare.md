# Preparation of Vibrato's dictionaries

This document describes steps to compile Vibrato's system dictionaries.

The following description assumes you are at the root directory of this repository.

## 1. Compiling system dictionary

You can compile system dictionaries from language resources in the [MeCab format](https://taku910.github.io/mecab/).
The simplest way is using publicly-available resources such as UniDic or IPADIC.

Here, consider to use `unidic-mecab-2.1.2`.

```
$ wget "https://clrd.ninjal.ac.jp/unidic_archive/cwj/2.1.2/unidic-mecab-2.1.2_src.zip" -O "./unidic-mecab-2.1.2_src.zip" --no-check-certificate
$ unzip unidic-mecab-2.1.2_src.zip
```

To compile the system dictionary from the resource,
run the following command.

```
$ cargo run --release -p prepare --bin system -- \
    -l unidic-mecab-2.1.2_src/lex.csv \
    -m unidic-mecab-2.1.2_src/matrix.def \
    -u unidic-mecab-2.1.2_src/unk.def \
    -c unidic-mecab-2.1.2_src/char.def \
    -o system.dic
```

## 2. Reordering mapping of connection ids

Vibrato supports faster tokenization by improving the locality of reference through mapping connection ids.

The reordering steps consist of
1. computing statistics from training data of sentences,
2. producing the reordered mapping, and
3. editing the system dictionary with the reordered mapping.

To produce the reordered mapping from sentences in `train.txt`,
run the following command.

```
$ cargo run --release -p prepare --bin reorder -- -i system.dic -o reordered < train.txt
```

The two files, `reordered.lmap` and `reordered.rmap`, will be produced.

## 3. Editing dictionary with mapping

To edit a system dictionary with the reordered mapping,
run the following command.

```
$ cargo run --release -p prepare --bin map -- -i system.dic -m reordered -o system.mapped.dic
```

When the matrix data is large,
`system.mapped.dic` will provide faster tokenization than `system.dic`.
