# Training dictionaries

This document describes steps to train dictionaries using Vibrato,
assuming you are in the root directory of this repository.

## 1. Training dictionary

To train a dictionary, you must prepare at least the following six files.

* `corpus.txt`: Corpus file to be trained. The format is the same as the output of the `tokenize` command of Vibrato.
                The contents of the feature columns must match exactly with the columns of the lexicon file.
                If it differs even slightly, it is considered an unknown word.
* `train_lex.csv`: Lexicon file to be weighted. All connection IDs and weights must be set to 0.
* `train_unk.def`: Unknown word file to be weighted. All connection IDs and weights must be set to 0.
* `char.def`: Character definition file.
* `rewrite.def`: Rewrite rule definition file.
* `feature.def`: Feature definition file.

The file formats follow those in MeCab (see the [official document](https://taku910.github.io/mecab/learn.html)).
You can also find an example dataset [here](../vibrato/src/tests/resources).

Execute the following command to start the training process (Replace file names with the actual ones):

```
$ cargo run --release -p train -- \
    -t ./dataset/corpus.txt \
    -l ./dataset/train_lex.csv \
    -u ./dataset/train_unk.def \
    -c ./dataset/char.def \
    -f ./dataset/feature.def \
    -r ./dataset/rewrite.def \
    -o ./modeldata.zst
```

The training command supports multi-threading and changing some parameters.
See the `--help` message for more details.

When training is complete, the model is output to `./modeldata.zst`.

## 2. Generating dictionary files

Run the following commands to generate a set of dictionary files from the model:

```
$ mkdir mydict # Prepare the output directory
$ cargo run --release -p dictgen -- \
    -i ./modeldata.zst \
    -l ./mydict/lex.csv \
    -u ./mydict/unk.def \
    -m ./mydict/matrix.def
```

Optionally, you can specify a user-defined dictionary to the `dictgen` command to automatically give connection IDs and weights.
See the `--help` message for more details.

After copying `dataset/char.def` under `mydict`, you can compile your system dictionary
following the [documentation](./compile.md).

## 3. Accuracy evaluation

You can evaluate the accuracy when using the trained dictionary. 

To split the input corpus randomly and output train/validation/test files, run the following command:

```
$ cargo run --release -p evaluate --bin split -- \
    -i ./dataset/corpus.txt \
    -t ./dataset/train.txt \
    -v ./dataset/valid.txt \
    -e ./dataset/test.txt
```

By default, 80% of the data is split into a training set, 10% into a validation set, and 10% into a test set.

To evaluate the accuracy, run the following command:

```
$ cargo run --release -p evaluate -- \
    -i ./system.dic.zst \
    -t ./dataset/valid.txt \
    --feature-indices 0,1,2,3,9
```

where `--feature-indices` is an option to specify features' indices to determine correctness.
In this example, the 0th, 1st, 2nd, 3rd, and 9th features are considered.
