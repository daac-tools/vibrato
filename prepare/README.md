# vibrato/prepare

This workspace provides several tools to compile Vibrato's dictionaries.

## 1. Compiling system dictionary

You can compile system dictionaries from language resources in the MeCab format.
The simplest way is using publicly-available resources such as UniDic or IPADIC.

Here, consider to use `unidic-mecab-2.1.2`.

```
$ wget "https://clrd.ninjal.ac.jp/unidic_archive/cwj/2.1.2/unidic-mecab-2.1.2_src.zip" -O "./unidic-mecab-2.1.2_src.zip" --no-check-certificate
$ unzip unidic-mecab-2.1.2_src.zip
```

To compile the system dictionary from the resource,
run the following command.

```
$ cargo run --release --bin system -- -r unidic-mecab-2.1.2_src -o system.dic
```

This command requires the four files, `lex.csv`, `matrix.def`, `char.def`, and `unk.def`, to be in the directory specified by `-r`.

## 2. Training mapping of connection ids

Vibrato supports faster tokenization by improving the locality of reference through mapping connection ids.

To tokenize sentences in `train.txt` and train the mapping,
run the following command.

```
$ cargo run --release --bin train -- -i system.dic -o trained < train.txt
```

The two files, `trained.lmap` and `trained.rmap`, will be produced.

## 3. Editing dictionary with mapping

To edit a system dictionary with the trained mapping,
run the following command.

```
$ cargo run --release --bin map -- -i system.dic -m trained -o system.mapped.dic
```

When the matrix data is large,
`system.mapped.dic` will provide faster tokenization then `system.dic`.
