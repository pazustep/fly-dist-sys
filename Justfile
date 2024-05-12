maelstrom := env_var_or_default('MAELSTROM_HOME', '~/devel/maelstrom') / "maelstrom"

[private]
default:
    @just --choose

_build:
    cargo build --release

echo: _build
    {{maelstrom}} test -w echo --bin target/release/echo --node-count 1 --time-limit 10

unique-ids: _build
    {{maelstrom}} test -w unique-ids --bin target/release/unique-ids --node-count 3 --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast: _build
    {{maelstrom}} test -w broadcast --bin target/release/broadcast --node-count 1 --time-limit 20 --rate 1000
