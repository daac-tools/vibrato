#!/usr/bin/env python3

import subprocess
import json
import os
import random
from argparse import ArgumentParser


NUM_FOLDS = 10
random.seed(42)


def compute_spans(N, K):
    spans = []
    i = 0
    for k in range(K):
        m = (k+N)//K
        spans.append((i, i+m))
        i += m
    return spans


def evaluate_nomap(args, test_lines):
    cmd_test = f'{args.exe_dir}/exp_timeperf -r {args.resource_dir}'
    subprocess.run(cmd_test, encoding='utf-8', shell=True, input='\n'.join(test_lines))


def evaluate_mapping(args, test_lines, train_lines):
    cmd_train = f'{args.exe_dir}/exp_mapping -r {args.resource_dir} -o tmp'
    subprocess.run(cmd_train, encoding='utf-8', shell=True, input='\n'.join(train_lines))

    cmd_test = f'{args.exe_dir}/exp_timeperf -r {args.resource_dir} -m tmp'
    subprocess.run(cmd_test, encoding='utf-8', shell=True, input='\n'.join(test_lines))

    os.remove('tmp.lmap')
    os.remove('tmp.rmap')


def main():
    parser = ArgumentParser()
    parser.add_argument('--exe_dir', '-e', type=str, required=True)
    parser.add_argument('--sent_file', '-s', type=str, required=True)
    parser.add_argument('--resource_dir', '-r', type=str, required=True)
    parser.add_argument('--with_mapping', '-m', action='store_true')
    args = parser.parse_args()

    print(args)

    lines = [l.rstrip() for l in open(args.sent_file, 'rt') if len(l.rstrip()) != 0]
    random.shuffle(lines)

    test_spans = compute_spans(len(lines), NUM_FOLDS)

    for k, (i, j) in enumerate(test_spans):
        print(f'** k={k} [{i},{j-1}] **')
        test_lines = lines[i:j]
        train_lines = lines[:i] + lines[j:]
        if args.with_mapping:
            evaluate_mapping(args, test_lines, train_lines)
        else:
            evaluate_nomap(args, test_lines)


if __name__ == "__main__":
    main()
