# Achieving faster tokenization through id mapping

This document describes steps to edit system dictionaries to achieve faster tokenization.
Here assumes that you have a system dictionary `system.dic`
produced in the manner described in [compile.md](./compile.md) and that 
you are at the root directory of this repository.

Vibrato supports faster tokenization by improving the locality of reference through mapping connection ids.
The mapping steps consist of
1. producing a reordered mapping using statistics obtained from training data of sentences, and
1. editing the system dictionary with the reordered mapping.

## 1. Reordering mapping of connection ids

To produce the reordered mapping from sentences in `train.txt`,
run the following command.

```
$ cargo run --release -p map --bin reorder -- -i system.dic -o reordered < train.txt
```

The two files, `reordered.lmap` and `reordered.rmap`, will be produced.

## 2. Editing dictionary with mapping

To edit a system dictionary with the reordered mapping,
run the following command.

```
$ cargo run --release -p map -- -i system.dic -m reordered -o system.mapped.dic
```

When the matrix data is large,
`system.mapped.dic` will provide faster tokenization than `system.dic`.
