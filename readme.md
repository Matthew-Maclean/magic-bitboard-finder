# magic-bitboard-finder

Finds magic bitboard numbers for rooks and bishops.

## How to Use

```
./magic-bitboard-finder -a=<ATTEMPTS> -e=<EXTRA> > out.rs
```

`ATTEMPTS` is the number of attempts each square gets before failing, in
millions. The default is 100 (= 100,000,000).

`EXTRA` is either `true` or `false`, if `true`, then once a square has reached
half the attempts without finding a magic number, an extra bit will be added
to the mask for that magic number, effectively doubling the hash space and
greatly increasing the likelyhood that a magic number will be found. Some
squares are very difficult to find without this (such as rook f8). The default
is `true`.

Info about the running of the program is printed to sterr. If successful, a
rust file with `static` arrays containing the magic numbers, offsets, and masks
will be printed to stdout.

## Example output

`out.rs` is an example output of the program invoked as:

```
./magic-bitboard-finder -a=1000 -e=true > out.rs
```

You can just take this file and use it if you don't want to make your own magic
numbers. In the example, only rook f8 required an extra bit to be found.
