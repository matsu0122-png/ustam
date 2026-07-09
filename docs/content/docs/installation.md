---
title: インストールと実行
weight: 20
description: ustam をビルド・実行する方法
---

ustam はRustのCargoプロジェクトです。

## cargo run で実行する

```bash
cargo run
```

実行すると、現在のディレクトリのファイル一覧が表示されます。

```text
Cargo.toml
LICENSE
README.md
src
```

## ビルドして実行する

毎回 `cargo run` を使わず、実行ファイルを作ってから起動することもできます。

```bash
cargo build
./target/debug/ustam
```

オプションやパスも同じように指定できます。

```bash
./target/debug/ustam -l src
```
