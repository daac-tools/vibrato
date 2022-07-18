# tiny-lattice

```
echo "京都東京都京都" | cargo run --release -p tokenize -- -d tinylattice/src/tests/resources/lex.csv -m tinylattice/src/tests/resources/matrix_10x10.def -c tinylattice/src/tests/resources/char.def -u tinylattice/src/tests/resources/unk.def
```

```
wget http://jaist.dl.sourceforge.net/project/mecab/mecab-ipadic/2.7.0-20070801/mecab-ipadic-2.7.0-20070801.tar.gz
tar -xzf mecab-ipadic-2.7.0-20070801.tar.gz
mv mecab-ipadic-2.7.0-20070801 mecab-ipadic
```

```
wget https://osdn.jp/projects/unidic/downloads/58338/unidic-mecab-2.1.2_src.zip
unzip unidic-mecab-2.1.2_src.zip
mv unidic-mecab-2.1.2_src unidic-mecab
```

```
echo "日本語の形態素解析を行うことができます。" | cargo run --release -p tokenize -- -r resources_mecab-ipadic
```