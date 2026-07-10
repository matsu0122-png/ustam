---
title: インストール
weight: 20
description: ustam をインストールする方法
---

## Homebrew

macOS / Linuxでは、Homebrewでインストールできます。bash/zsh/fishの補完スクリプトも自動的に配置されます。

```bash
brew install matsu0122-png/ustam/ustam
```

## Docker

コンテナイメージとしても配布しています。カレントディレクトリをマウントして実行できます。

```bash
docker run --rm -v "$(pwd)":/workspace ghcr.io/matsu0122-png/ustam -l
```

## ソースからビルドする

Rustのツールチェインがあれば、ソースからビルドできます。

```bash
git clone https://github.com/matsu0122-png/ustam.git
cd ustam
cargo build --release
```

生成された `target/release/ustam` を `PATH` の通った場所へ配置してください。
