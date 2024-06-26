# Zataxx - Ataxx engine in Rust

# How to compile

```cargo rustc --release -- -C target-cpu=native```

The exe will be in `target/release`

# UAI (Universal Ataxx Interface)

### Options

- Hash (int, default 32, 1 to 1024) - transposition table size in MB

### Extra commands

- display

- eval

- perft \<depth\>

- perftsplit \<depth\>

- bench \<depth\>

# Features

### Board
- Bitboards
- Copymake make/undo move
- Zobrist hashing

### Evaluation
- 2-tuples Neural Network (2916->256->1)
- Self-play data
- SCReLU activation

### Search
- Iterative deepening
- Aspiration windows
- Fail-soft negamax
- Principal variation search
- Transposition table
- Alpha-beta pruning
- Reverse futility pruning
- Late move pruning
- Futility pruning
- Multicut
- Internal iterative reduction
- Late move reductions
- Singular extension, negative extension
- Move ordering: TT move then most captures, equal captures ordered by history

### Time management
- Soft and hard limits
- Nodes TM

# Credits

[bullet](https://github.com/jw1912/bullet) for training the NN
