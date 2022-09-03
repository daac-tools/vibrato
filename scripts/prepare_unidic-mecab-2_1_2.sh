#!/bin/bash

set -eux

type wget
type unzip
type sort
type openssl

corpus_name="unidic-mecab-2_1_2"
resources_dir="resources_${corpus_name}"

if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit 1
fi

# Builds the system dictionary.
wget --timeout 3 -t 10 "https://clrd.ninjal.ac.jp/unidic_archive/cwj/2.1.2/unidic-mecab-2.1.2_src.zip" -O "./unidic-mecab-2.1.2_src.zip" --no-check-certificate
if [ $? -ne 0 ]; then
  echo "[ERROR] Failed to download the resource. Please retry later."
  exit 1
fi
unzip unidic-mecab-2.1.2_src.zip

mkdir ${resources_dir}
env LC_ALL=C cat unidic-mecab-2.1.2_src/lex.csv | sort > ${resources_dir}/lex.csv
mv unidic-mecab-2.1.2_src/char.def ${resources_dir}/char.def
mv unidic-mecab-2.1.2_src/unk.def ${resources_dir}/unk.def
mv unidic-mecab-2.1.2_src/matrix.def ${resources_dir}/matrix.def

rm -rf unidic-mecab-2.1.2_src
rm -f unidic-mecab-2.1.2_src.zip

cargo run --release -p prepare --bin system -- -r ${resources_dir} -o ${resources_dir}/system.dic

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
cargo run --release -p prepare --bin train -- -i ${resources_dir}/system.dic -o ${resources_dir}/kftt < kftt-data-1.0/data/orig/kyoto-train.ja
rm -rf kftt-data-1.0

# Maps ids
cargo run --release -p prepare --bin map -- -i ${resources_dir}/system.dic -m ${resources_dir}/kftt -o ${resources_dir}/system.dic

# Removes unnecessary data
rm -f ${resources_dir}/lex.csv ${resources_dir}/char.def ${resources_dir}/unk.def ${resources_dir}/matrix.def ${resources_dir}/kftt.lmap ${resources_dir}/kftt.rmap
