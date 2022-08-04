#!/bin/bash

set -eux

which wget
which unzip
which sort

corpus_name="unidic-cwj-3_1_0"
resources_dir="resources_"${corpus_name}

if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit
fi

wget "https://clrd.ninjal.ac.jp/unidic_archive/cwj/3.1.0/unidic-cwj-3.1.0-full.zip" -O "./unidic-cwj-3.1.0-full.zip" --no-check-certificate
unzip unidic-cwj-3.1.0-full.zip

mkdir ${resources_dir}
env LC_ALL=C cat unidic-cwj-3.1.0-full/lex_3_1.csv | sort > ${resources_dir}/lex.csv
mv unidic-cwj-3.1.0-full/char.def ${resources_dir}/
mv unidic-cwj-3.1.0-full/unk.def ${resources_dir}/
mv unidic-cwj-3.1.0-full/matrix.def ${resources_dir}/

rm -rf unidic-cwj-3.1.0-full
rm -f unidic-cwj-3.1.0-full.zip

cargo run --release -p prepare --bin system -- -r ${resources_dir} -o ${resources_dir}/system.dic
cargo run --release -p prepare --bin map -- -i ${resources_dir}/system.dic -m data/mappings/${corpus_name} -o ${resources_dir}/system.dic

rm -f ${resources_dir}/lex.csv ${resources_dir}/char.def ${resources_dir}/unk.def ${resources_dir}/matrix.def
