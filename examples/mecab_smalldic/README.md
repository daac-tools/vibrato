# Small dictionary generator for MeCab dictionary

This tool allows for generating a smaller dictionary from a pre-trained MeCab model
without the training steps described [`small-dic.md`](../../docs/small-dic.md).
## Usage

This tool needs several files that describe the definitions of your pre-trained model.
(See the tool's help for more information.)

The following shows an example of using [UniDic 3.1.1](https://clrd.ninjal.ac.jp/unidic/).
(`scrape.py` is a patch file for compatibility.)
```
$ wget https://clrd.ninjal.ac.jp/unidic_archive/cwj/3.1.1/unidic-cwj-3.1.1-full.zip
$ unzip unidic-cwj-3.1.1-full.zip
$ cargo run --release -- \
    -l <(python3 scrape.py < unidic-cwj-3.1.1-full/lex_3_1.csv) \
    -u <(python3 scrape.py < unidic-cwj-3.1.1-full/unk.def) \
    -c unidic-cwj-3.1.1-full/char.def \
    -f <(sed -e 's/ F_F .*$/ F_F\/F_F/' -e 's/ I_I .*$/ I_I\/I_I/' < unidic-cwj-3.1.1-full/feature.def) \
    -a unidic-cwj-3.1.1-full/right-id.def \
    -b unidic-cwj-3.1.1-full/left-id.def \
    -m <(sed -e 's/\tF_F$/\tF_F\/F_F/' -e 's/\tI_I$/\tI_I\/I_I/' < unidic-cwj-3.1.1-full/model.def) \
    -r 700 \
    -o ./small.dic.zst
```
