#!/bin/bash

set -eux

resources_names=(
    "ipadic-mecab-2_7_0"
    "unidic-mecab-2_1_2"
    "unidic-cwj-3_1_0"
)
logs_dir=$1

for resources_name in "${resources_names[@]}" ; do
    ./runners/bccwj.sh resources_${resources_name} &> ${logs_dir}/logs_${resources_name}.txt
done

for resources_name in "${resources_names[@]}" ; do
    ./runners/bccwj-ideal.sh resources_${resources_name} &> ${logs_dir}/logs_${resources_name}-ideal.txt
done
