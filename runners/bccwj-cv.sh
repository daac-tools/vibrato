#!/bin/bash

src="bccwj-texts-10k"

set -eux

files=(
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

for file in "${files[@]}" ; do
    ./runners/cv.py -e target/release -s ${src}/${file} -r $1
    ./runners/cv.py -e target/release -s ${src}/${file} -r $1 -m
done