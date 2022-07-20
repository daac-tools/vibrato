#!/bin/bash

set -eux

which wget
which unzip
which sort

resources_dir="resources_unidic-mecab-2_1_2"

if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit
fi

wget "https://clrd.ninjal.ac.jp/unidic_archive/cwj/2.1.2/unidic-mecab-2.1.2_src.zip" -O "./unidic-mecab-2.1.2_src.zip" --no-check-certificate
unzip unidic-mecab-2.1.2_src.zip

mkdir ${resources_dir}
env LC_ALL=C cat unidic-mecab-2.1.2_src/lex.csv | sort > ${resources_dir}/lex.csv
mv unidic-mecab-2.1.2_src/char.def ${resources_dir}/
mv unidic-mecab-2.1.2_src/unk.def ${resources_dir}/
mv unidic-mecab-2.1.2_src/matrix.def ${resources_dir}/

rm -rf unidic-mecab-2.1.2_src
rm unidic-mecab-2.1.2_src.zip
