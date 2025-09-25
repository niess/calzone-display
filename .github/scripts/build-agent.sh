#! /bin/bash
manifest="crates/Cargo.toml"
agent="calzone-display-agent"
src="crates/target/release"
dst="calzone_display/.bins"

cargo build --manifest-path=${manifest} -p ${agent} --release
mkdir -p ${dst}
mv ${src}/${agent} ${dst}
cargo clean --manifest-path=${manifest}
