#!/bin/bash

set -eux

exe_dir="target/release"
resources_dir=$1
train_file="bccwj-texts-10k/CORE.txt"
test_dir="bccwj-texts-10k"
test_files=(
    "LB.txt"
    "OB.txt"
    "OC.txt"
    "OL.txt"
    "OM.txt"
    "OP.txt"
    "OT.txt"
    "OV.txt"
    "OW.txt"
    "OY.txt"
    "PB.txt"
    "PM.txt"
    'PN.txt'
)
map_file="CORE"

./${exe_dir}/exp_mapping -r ${resources_dir} -o ${map_file} < ${train_file}

for test_file in "${test_files[@]}" ; do
    ./runners/evaluate.py -e ${exe_dir} -s ${test_dir}/${test_file} -r ${resources_dir}
    ./runners/evaluate.py -e ${exe_dir} -s ${test_dir}/${test_file} -r ${resources_dir} -m ${map_file}
    ./runners/evaluate.py -e ${exe_dir} -s ${test_dir}/${test_file} -r ${resources_dir} -M
done

rm ${map_file}.lmap
rm ${map_file}.rmap
