#!/bin/bash

set -eux

type wget
type tar
type iconv
type sort
type openssl

corpus_name="ipadic-mecab-2_7_0"
resources_dir="resources_${corpus_name}"

if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit 1
fi

# Builds the system dictionary.
wget --timeout 3 -t 10 http://jaist.dl.sourceforge.net/project/mecab/mecab-ipadic/2.7.0-20070801/mecab-ipadic-2.7.0-20070801.tar.gz
if [ $? -ne 0 ]; then
  echo "[ERROR] Failed to download the resource. Please retry later."
  exit 1
fi

tar -xzf mecab-ipadic-2.7.0-20070801.tar.gz

mkdir ${resources_dir}
env LC_ALL=C cat mecab-ipadic-2.7.0-20070801/*.csv | iconv -f EUCJP -t UTF8 | sort > ${resources_dir}/lex.csv
cat mecab-ipadic-2.7.0-20070801/char.def | iconv -f EUCJP -t UTF8 > ${resources_dir}/char.def
cat mecab-ipadic-2.7.0-20070801/unk.def | iconv -f EUCJP -t UTF8 > ${resources_dir}/unk.def
mv mecab-ipadic-2.7.0-20070801/matrix.def ${resources_dir}/matrix.def

rm -rf mecab-ipadic-2.7.0-20070801
rm -f mecab-ipadic-2.7.0-20070801.tar.gz

cargo run --release -p prepare --bin system -- -l ${resources_dir}/lex.csv -m ${resources_dir}/matrix.def -u ${resources_dir}/unk.def -c ${resources_dir}/char.def -o ${resources_dir}/system.dic

# Trains the mapping
if [ ! -e kftt-data-1.0.tar.gz ]; then
  wget --timeout 3 -t 10 http://www.phontron.com/kftt/download/kftt-data-1.0.tar.gz
  if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to download the resource. Please retry later."
    exit 1
  fi
else
  echo "kftt-data-1.0.tar.gz is already there."
fi

tmp_hash=`openssl sha1 kftt-data-1.0.tar.gz | cut -d $' ' -f 2,2`
if [ "${tmp_hash}" != "0e1f5a9dc993b7d74ca6a0521232d17ce94c8cb4" ]; then
  echo "[ERROR] Hash value of kftt-data-1.0.tar.gz doesn't match."
  exit 1;
fi

tar -xzf kftt-data-1.0.tar.gz
cargo run --release -p prepare --bin reorder -- -i ${resources_dir}/system.dic -o ${resources_dir}/kftt < kftt-data-1.0/data/orig/kyoto-train.ja
rm -rf kftt-data-1.0

# Maps ids
cargo run --release -p prepare --bin map -- -i ${resources_dir}/system.dic -m ${resources_dir}/kftt -o ${resources_dir}/system.dic

# Removes unnecessary data
rm -f ${resources_dir}/lex.csv ${resources_dir}/char.def ${resources_dir}/unk.def ${resources_dir}/matrix.def ${resources_dir}/kftt.lmap ${resources_dir}/kftt.rmap
