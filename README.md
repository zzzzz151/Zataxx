# Zataxx - Ataxx engine in Rust

# How to compile

```cargo rustc --release -- -C target-cpu=native```

The exe will be in `target/release`

# UAI (Universal Ataxx Interface)

### Options

- Hash (int, default 32, 1 to 1024) - transposition table size in MB

### Extra commands

- eval - displays current position's evaluation from perspective of side to move

- perft \<depth\> - runs perft from current position

- perftsplit \<depth\> - runs split perft from current position

- bench \<depth\> - runs bench

# Features

### Board
- Bitboards
- Make/undo move
- Zobrist hashing

### NNUE evaluation
- (98->512)x2->1
- Self-play data
- SCReLU activation

### Search
- Iterative deepening
- Principal variation search with fail-soft Negamax
- Transposition table
- Alpha-beta pruning
- Reverse futility pruning
- Late move pruning
- Futility pruning
- Multicut
- Internal iterative reduction
- Late move reductions
- Singular extension, negative extension
- Move ordering: TT move -> singles by captures -> doubles by captures
- Bonus for killer move

### Time management
- Soft and hard limits
- Nodes TM

# Credits

[bullet](https://github.com/jw1912/bullet) for training the NN
