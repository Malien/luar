set -e

cd lex
cargo t --features quickcheck
cd ..

cd syn
cargo t --features quickcheck
cd ..

cd reggie
cargo t --features quickcheck
cd ..

cargo test -p ast_vm --lib
cargo test --test reggie
cargo test --test engine
cargo test --test correctness
