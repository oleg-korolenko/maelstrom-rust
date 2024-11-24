# maelstrom-rust

[Fly.io distributed system challenges](https://fly.io/dist-sys/1/)  to play with Rust

## To test

Run from your maelstrom installation directory

### echo

```sh
./maelstrom test -w unique-ids --bin ../maelstrom-rust/target/debug/unique-id --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition --log-stderr
```

### unique-id

```sh
./maelstrom test -w unique-ids --bin ../maelstrom-rust/target/debug/unique-id --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition --log-stderr

```

### broadcast

#### single-node

```sh
./maelstrom test -w broadcast --bin ../maelstrom-rust/target/debug/broadcast --node-count 1 --time-limit 20 --rate 10 --availability total --nemesis partition --log-stderr

```

### multi-node

```sh  
./maelstrom test -w broadcast --bin ../maelstrom-rust/target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --availability total --nemesis partition --log-stderr
```
./maelstrom test -w broadcast --bin ../maelstrom-rust/target/debug/broadcast --node-count 5 --time-limit 20 --rate 10