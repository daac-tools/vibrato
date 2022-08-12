#!/bin/bash

set -eux

which git
which wget
which tar
which iconv
which sort
which openssl

# Edit these if you want to download another version.
ymd="20200910"
commitid="abc61e3"

corpus_name="ipadic-mecab-neologd-${ymd}"
resources_dir="resources_${corpus_name}"
workspace_dir="workspace_${corpus_name}"

if [ -d ${workspace_dir} ]; then
  echo "Directory ${workspace_dir} already exits."
  exit 1
fi
if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit 1
fi

# Builds the system dictionary.
mkdir ${workspace_dir}
pushd ${workspace_dir}
  git clone https://github.com/neologd/mecab-ipadic-neologd.git
  pushd mecab-ipadic-neologd
    git checkout ${commitid}
    ./libexec/make-mecab-ipadic-neologd.sh
  popd
popd

target_resources_dir="${workspace_dir}/mecab-ipadic-neologd/build/mecab-ipadic-2.7.0-20070801-neologd-${ymd}"

mkdir ${resources_dir}
env LC_ALL=C cat ${target_resources_dir}/*.csv | sort >| ${resources_dir}/lex.csv
mv ${target_resources_dir}/matrix.def ${resources_dir}/matrix.def
mv ${target_resources_dir}/char.def ${resources_dir}/char.def
mv ${target_resources_dir}/unk.def ${resources_dir}/unk.def
rm -rf ${workspace_dir}

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