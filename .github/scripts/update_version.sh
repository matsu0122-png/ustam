#! /bin/sh

FROM_VERSION=`grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/g'`
TO_VERSION=$1

sed "s/^version = \".*\"/version = \"$TO_VERSION\"/" Cargo.toml > a ; mv a Cargo.toml
sed "s/\${VERSION}/$TO_VERSION/g" .github/templates/README.md > a ; mv a README.md

# Cargo.toml のバージョンだけでなく Cargo.lock も更新しておかないと、
# `cargo build --locked`（Containerfile 等）がロックファイル不一致で失敗する。
cargo update -p ustam --precise "$TO_VERSION"