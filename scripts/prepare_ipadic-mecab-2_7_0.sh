#!/bin/bash

set -eux

which wget
which tar
which iconv
which sort

resources_dir="resources_ipadic-mecab-2_7_0"

if [ -d ${resources_dir} ]; then
  echo "Directory ${resources_dir} already exits."
  exit
fi

wget http://jaist.dl.sourceforge.net/project/mecab/mecab-ipadic/2.7.0-20070801/mecab-ipadic-2.7.0-20070801.tar.gz
tar -xzf mecab-ipadic-2.7.0-20070801.tar.gz

mkdir ${resources_dir}
env LC_ALL=C cat mecab-ipadic-2.7.0-20070801/*.csv | iconv -f EUCJP -t UTF8 | sort > ${resources_dir}/lex.csv
cat mecab-ipadic-2.7.0-20070801/char.def | iconv -f EUCJP -t UTF8 > ${resources_dir}/char.def
cat mecab-ipadic-2.7.0-20070801/unk.def | iconv -f EUCJP -t UTF8 > ${resources_dir}/unk.def
mv mecab-ipadic-2.7.0-20070801/matrix.def ${resources_dir}/

rm -rf mecab-ipadic-2.7.0-20070801
rm -f mecab-ipadic-2.7.0-20070801.tar.gz
