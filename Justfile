maelstrom := env_var_or_default('MAELSTROM_HOME', '~/devel/maelstrom') / "maelstrom"

[private]
default:
    @just --choose

_build package:
    cargo build -p {{package}} --release

echo: (_build "echo")
    {{maelstrom}} test -w echo --bin target/release/echo --node-count 1 --time-limit 10

unique-ids: (_build "unique-ids")
    {{maelstrom}} test -w unique-ids --bin target/release/unique-ids --node-count 3 --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast-single: (_build "broadcast")
    {{maelstrom}} test -w broadcast --bin target/release/broadcast --node-count 1 --time-limit 20 --rate 10

broadcast-multi: (_build "broadcast")
    {{maelstrom}} test -w broadcast --bin target/release/broadcast --node-count 5 --time-limit 20 --rate 10

serve:
    {{maelstrom}} serve
