#!/bin/bash

set -eux

compiled_dir="resources_compiled"
train_file="bccwj-texts-10k/CORE.txt"
test_file="wagahaiwa_nekodearu.txt"
resource_names=(
    "ipadic-mecab-2_7_0"
    "unidic-mecab-2_1_2"
    "unidic-cwj-3_1_0"
)

mkdir ${compiled_dir}

for resource_name in "${resource_names[@]}" ; do
    cargo run --release -p compile -- -r resources_${resource_name} -o ${compiled_dir}/${resource_name}.dic
    cargo run --release -p benchmark -- -i ${compiled_dir}/${resource_name}.dic < ${test_file}
    cargo run --release -p compile -- -r resources_${resource_name} -t ${train_file} -o ${compiled_dir}/${resource_name}.train.dic
    cargo run --release -p benchmark -- -i ${compiled_dir}/${resource_name}.train.dic < ${test_file}
done
