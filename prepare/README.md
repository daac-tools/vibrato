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

## 2. Reordering mapping of connection ids

Vibrato supports faster tokenization by improving the locality of reference through mapping connection ids.

To tokenize sentences in `reorder.txt` and reorder the mapping,
run the following command.

```
$ cargo run --release --bin reorder -- -i system.dic -o reordered < reorder.txt
```

The two files, `reordered.lmap` and `reordered.rmap`, will be produced.

## 3. Editing dictionary with mapping

To edit a system dictionary with the reordered mapping,
run the following command.

```
$ cargo run --release --bin map -- -i system.dic -m reordered -o system.mapped.dic
```

When the matrix data is large,
`system.mapped.dic` will provide faster tokenization then `system.dic`.
