#! /bin/bash
agent="calzone-display-agent"
src="target/release"
dst="calzone_display/.bins"

cargo build -p ${agent} --release
mkdir -p ${dst}
mv ${src}/${agent} ${dst}
cargo clean
