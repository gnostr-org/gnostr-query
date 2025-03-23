default:
    @just --choose
run:
    @cargo -q run --bin gnostr-query -- -t gnostr -l 500
install:
    @cargo install --bin gnostr-query --path .
