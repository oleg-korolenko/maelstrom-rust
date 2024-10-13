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
