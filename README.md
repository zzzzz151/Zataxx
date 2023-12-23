# Zataxx - Ataxx engine in Rust

# How to compile

```cargo build --release```

The exe will be in target/release.

# UAI (Universal Ataxx Interface)

### Options

- Hash (int, default 32, 1 to 1024) - transposition table size in MB

### Extra commands

- eval - displays current position's evaluation from perspective of side to move

- perft \<depth\> - runs perft from current position

- perftsplit \<depth\> - runs split perft from current position

- gameresult - displays current game result ("*" not over, "1.0" red won, "0.0" red lost, "0.5" draw)

# Features

### Board
- Bitboards
- Make/undo move
- Zobrist hashing

### NNUE evaluation
- 147->128x2->1
- Self-play data
- Screlu activation

### Search framework
- Iterative deepening
- Principal variation search with fail-soft Negamax
- Transposition table

### Move ordering
- TT move
- Bonus for single moves
- Most captures

### Pruning
- Alpha-beta pruning
- Reverse futility pruning
- Late move pruning
- Futility pruning

### Reductions
- Late move reductions

### Time management
- Soft and hard limits
- Soft limit scaling based on best move nodes

# Credits

[bullet](https://github.com/jw1912/bullet) for training the NN
