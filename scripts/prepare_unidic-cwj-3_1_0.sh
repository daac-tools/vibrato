#!/bin/bash

set -eux

which wget
which unzip
which sort

resources_dir="resources_unidic-cwj-3_1_0"

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
rm -rf unidic-cwj-3.1.0-full.zip

cargo run --release -p prepare --bin system -- -r resources_unidic-cwj-3_1_0 -o unidic-cwj-3_1_0.dic
cargo run --release -p prepare --bin map -- -i unidic-cwj-3_1_0.dic -m data/mappings/unidic-cwj-3_1_0 -o unidic-cwj-3_1_0.dic
