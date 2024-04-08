#!/usr/bin/env sh
echo build
cargo build --release
echo move picture to target folder
cp picturetest.png target/release
