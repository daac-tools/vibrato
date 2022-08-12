#!/bin/bash

set -eux

which wget
which tar
which iconv
which sort

corpus_name="ipadic-mecab-2_7_0"
resources_dir="resources_${corpus_name}"

if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit
fi

# Builds the system dictionary.
wget http://jaist.dl.sourceforge.net/project/mecab/mecab-ipadic/2.7.0-20070801/mecab-ipadic-2.7.0-20070801.tar.gz
tar -xzf mecab-ipadic-2.7.0-20070801.tar.gz

mkdir ${resources_dir}
env LC_ALL=C cat mecab-ipadic-2.7.0-20070801/*.csv | iconv -f EUCJP -t UTF8 | sort > ${resources_dir}/lex.csv
cat mecab-ipadic-2.7.0-20070801/char.def | iconv -f EUCJP -t UTF8 > ${resources_dir}/char.def
cat mecab-ipadic-2.7.0-20070801/unk.def | iconv -f EUCJP -t UTF8 > ${resources_dir}/unk.def
mv mecab-ipadic-2.7.0-20070801/matrix.def ${resources_dir}/matrix.def

rm -rf mecab-ipadic-2.7.0-20070801
rm -f mecab-ipadic-2.7.0-20070801.tar.gz

cargo run --release -p prepare --bin system -- -r ${resources_dir} -o ${resources_dir}/system.dic

# Trains the mapping
wget http://www.phontron.com/kftt/download/kftt-data-1.0.tar.gz
tar -xzf kftt-data-1.0.tar.gz

cargo run --release -p prepare --bin train -- -i ${resources_dir}/system.dic -o ${resources_dir}/kftt < kftt-data-1.0/data/orig/kyoto-train.ja

rm -rf kftt-data-1.0
rm -f kftt-data-1.0.tar.gz

# Maps ids
cargo run --release -p prepare --bin map -- -i ${resources_dir}/system.dic -m ${resources_dir}/kftt -o ${resources_dir}/system.dic

# Removes unnecessary data
rm -f ${resources_dir}/lex.csv ${resources_dir}/char.def ${resources_dir}/unk.def ${resources_dir}/matrix.def ${resources_dir}/kftt.lmap ${resources_dir}/kftt.rmap
