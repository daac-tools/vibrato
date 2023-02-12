# Small dictionary generator for MeCab dictionary

## Usage

```
$ wget https://clrd.ninjal.ac.jp/unidic_archive/cwj/3.1.1/unidic-cwj-3.1.1.zip
$ unzip unidic-cwj-3.1.1.zip
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
