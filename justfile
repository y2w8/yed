build mode="":
  cargo build --{{mode}}

run file="justfile":
  cargo run file

deploy:
  cargo build --release
