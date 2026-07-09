# ustam

![build](https://img.shields.io/github/actions/workflow/status/matsu0122-png/ustam/build.yaml?branch=main&label=build)
![update version](https://img.shields.io/github/actions/workflow/status/matsu0122-png/ustam/update_version.yaml?label=update%20version)
![publish](https://img.shields.io/github/actions/workflow/status/matsu0122-png/ustam/publish.yaml?label=publish)
![Coverage Status](https://img.shields.io/coverallsCoverage/github/matsu0122-png/ustam?branch=main)
![License](https://img.shields.io/github/license/matsu0122-png/ustam)
![Release](https://img.shields.io/github/v/release/matsu0122-png/ustam)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.21275228.svg)](https://doi.org/10.5281/zenodo.21275228)

## Tagline
Rustで実装した拡張版 `ls` コマンド風CLIツール

## 概要
このプログラムは、指定したディレクトリ内のファイルやディレクトリ一覧を表示するCLIツールです。
パスを指定しない場合は、現在のディレクトリの内容を表示します。

通常の `ls` のような一覧表示に加えて、詳細表示、並び替え、`.gitignore` を考慮した表示制御、READMEやPDFからの追加情報表示に対応しています。

## 主な機能
- 現在のディレクトリ、または指定したディレクトリの一覧表示
- 隠しファイルの表示切り替え
- ファイル種別、サイズ、更新時刻を含む詳細表示
- ファイルサイズ、更新日時、名前によるソート
- `.gitignore` に書かれたファイルやディレクトリの除外
- ディレクトリ内の `README.md` からTaglineを読み取り表示
- PDFファイルのタイトル情報を読み取り表示

## 実行方法
このプロジェクトはRustのCargoプロジェクトです。
以下のコマンドで実行できます。

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

## 使い方
基本形式は以下です。

```bash
cargo run -- [OPTIONS] [PATH]
```

`PATH` を省略すると、現在のディレクトリを対象にします。

### 例
現在のディレクトリを表示します。

```bash
cargo run
```

`src` ディレクトリの中身を表示します。

```bash
cargo run -- src
```

隠しファイルも表示します。

```bash
cargo run -- -a
```

詳細形式で表示します。

```bash
cargo run -- -l
```

サイズ順に並び替えます。

```bash
cargo run -- -s
```

更新日時順に並び替えます。

```bash
cargo run -- -t
```

複数のオプションを組み合わせることもできます。

```bash
cargo run -- -al .
```

## オプション
| オプション | 内容 |
| --- | --- |
| `-a` | 隠しファイルを表示する |
| `-l` | サイズ、更新日時、追加情報を表示する |
| `-s` | ファイルサイズ順にソートする |
| `-t` | 更新日時順にソートする |
| `-n` | 名前順にソートする |
| `-h` | ヘルプを表示する |

## ビルドして実行する方法
毎回 `cargo run` を使わず、実行ファイルを作ってから起動することもできます。

```bash
cargo build
./target/debug/ustam
```

オプションやパスも同じように指定できます。

```bash
./target/debug/ustam -l src
```
