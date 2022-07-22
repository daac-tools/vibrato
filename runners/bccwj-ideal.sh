#!/bin/bash

set -eux

resources_dir=$1
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

cargo build --release -p exp_timeperf --features exp-ideal

for test_file in "${test_files[@]}" ; do
    ./target/release/exp_timeperf -r ${resources_dir} < ${test_dir}/${test_file}
done
