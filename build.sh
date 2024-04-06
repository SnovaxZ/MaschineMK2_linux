#!/usr/bin/env sh
echo build
cargo build --release
echo move picture to target folder
mv picturetest.png target/release
