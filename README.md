# tiny-lattice

## Resource preparation

```
$ prepare_ipadic-mecab-2_7_0.sh
$ prepare_unidic-cwj-3_1_0.sh
$ prepare_unidic-mecab-2_1_2.sh
```

## Compiling dictionary

```
$ cargo run --release -p compile -- -r resources_ipadic-mecab-2_7_0 -o system.dic
Compiling the system dictionary...
1.485708241 seconds
Writting the system dictionary...
44.636895179748535 MiB
```

## Benchmark

```
$ cargo run --release -p benchmark -- -i system.dic < wagahaiwa_nekodearu.txt
    Finished release [optimized] target(s) in 0.04s
     Running `target/release/benchmark -i system.dic`
[benchmark/src/main.rs:50] n_words = 2462700
Warmup: 0.1255080327
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
[benchmark/src/main.rs:50] n_words = 2462700
Number_of_sentences: 2376
Elapsed_seconds_to_tokenize_all_sentences: [0.12153107410000001,0.12437256297500002,0.1282961128]
```
