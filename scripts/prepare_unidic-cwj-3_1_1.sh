#!/bin/bash

set -eux

type wget
type unzip
type sort
type openssl

corpus_name="unidic-cwj-3_1_1"
resources_dir="resources_${corpus_name}"

if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit 1
fi

# Builds the system dictionary.
wget --timeout 3 -t 10 "https://clrd.ninjal.ac.jp/unidic_archive/cwj/3.1.1/unidic-cwj-3.1.1-full.zip" -O "./unidic-cwj-3.1.1-full.zip" --no-check-certificate
if [ $? -ne 0 ]; then
  echo "[ERROR] Failed to download the resource. Please retry later."
  exit 1
fi
unzip unidic-cwj-3.1.1-full.zip

mkdir ${resources_dir}
env LC_ALL=C cat unidic-cwj-3.1.1-full/lex_3_1.csv | sort > ${resources_dir}/lex.csv
mv unidic-cwj-3.1.1-full/char.def ${resources_dir}/
mv unidic-cwj-3.1.1-full/unk.def ${resources_dir}/
mv unidic-cwj-3.1.1-full/matrix.def ${resources_dir}/

rm -rf unidic-cwj-3.1.1-full
rm -f unidic-cwj-3.1.1-full.zip

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
cargo run --release -p prepare --bin reorder -- -i ${resources_dir}/system.dic -o ${resources_dir}/kftt < kftt-data-1.0/data/orig/kyoto-train.ja
rm -rf kftt-data-1.0

# Maps ids
cargo run --release -p prepare --bin map -- -i ${resources_dir}/system.dic -m ${resources_dir}/kftt -o ${resources_dir}/system.dic

# Removes unnecessary data
rm -f ${resources_dir}/lex.csv ${resources_dir}/char.def ${resources_dir}/unk.def ${resources_dir}/matrix.def ${resources_dir}/kftt.lmap ${resources_dir}/kftt.rmap
