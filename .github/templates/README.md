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

## Overview（概要）
このプログラムは、指定したディレクトリ内のファイルやディレクトリ一覧を表示するCLIツールです。
パスを指定しない場合は、現在のディレクトリの内容を表示します。

通常の `ls` のような一覧表示に加えて、詳細表示、並び替え、`.gitignore` を考慮した表示制御、READMEやPDFからの追加情報表示に対応しています。

### 主な機能
- 現在のディレクトリ、または指定したディレクトリの一覧表示
- 隠しファイルの表示切り替え
- ファイル種別、サイズ、更新時刻を含む詳細表示
- ファイルサイズ、更新日時、名前によるソート
- `.gitignore` に書かれたファイルやディレクトリの除外
- ディレクトリ内の `README.md` からTaglineを読み取り表示
- PDFファイルのタイトル情報を読み取り表示
- シェル補完スクリプトの生成（bash/zsh/fish/powershell/elvish）

## Installation（インストール）

### Homebrew
macOS / Linuxでは、Homebrewでインストールできます。補完スクリプトも自動配置されます。

```bash
brew install matsu0122-png/ustam/ustam
```

### ソースからビルド
このプロジェクトはRustのCargoプロジェクトです。

```bash
git clone https://github.com/matsu0122-png/ustam.git
cd ustam
cargo build --release
```

生成された `target/release/ustam` を `PATH` の通った場所へ配置してください。

### Docker
`Containerfile` からコンテナイメージをビルドし、カレントディレクトリをマウントして実行できます。

```bash
docker build -t ustam -f Containerfile .
docker run --rm -v "$(pwd)":/workspace ustam -l
```

`just` がインストールされていれば `just docker-build` / `just docker-run -l` でも同じことができます。
ビルド済みイメージは `ghcr.io/matsu0122-png/ustam` としても配布しています。

## Usage（使い方）
基本形式は以下です。

```bash
ustam [OPTIONS] [PATH]
```

`PATH` を省略すると、現在のディレクトリを対象にします。
開発中でインストール前に試す場合は `ustam` の代わりに `cargo run --` を使ってください。

### オプション
| オプション | 内容 |
| --- | --- |
| `-a`, `--all` | 隠しファイルを表示する |
| `-l`, `--long` | サイズ、更新日時、追加情報を表示する |
| `-s`, `--size` | ファイルサイズ順にソートする |
| `-t`, `--time` | 更新日時順にソートする |
| `-n`, `--name` | 名前順にソートする（デフォルト） |
| `--completions <SHELL>` | 指定したシェル向けの補完スクリプトを標準出力へ書き出す（bash/zsh/fish/powershell/elvish） |
| `-h`, `--help` | ヘルプを表示する |
| `-V`, `--version` | バージョンを表示する |

`-s`/`-t`/`-n` は互いに排他的で、同時に指定するとエラーになります。

### シェル補完
`--completions` オプションで、シェルの補完スクリプトを生成できます。

```bash
ustam --completions bash > /usr/local/etc/bash_completion.d/ustam
ustam --completions zsh  > /usr/local/share/zsh/site-functions/_ustam
```

## Examples（例）
現在のディレクトリを表示します。

```bash
ustam
```

`src` ディレクトリの中身を表示します。

```bash
ustam src
```

隠しファイルも表示します。

```bash
ustam -a
```

詳細形式で、サイズの大きい順に表示します。

```bash
ustam -sl
```

複数のオプションを組み合わせることもできます。

```bash
ustam -al .
```

## About
`ustam` は、大学の講義「エンピリカルソフトウェア工学」の課題として開発されたプロジェクトです。
単にCLIツールとして動作するだけでなく、テスト、CI/CD、Docker対応、リリース自動化、パッケージ配布（Homebrew）、ドキュメント整備といったソフトウェア工学のプラクティスを実践することを目的としています。

エンドユーザー向けの詳細ドキュメントは [ドキュメントサイト](https://matsu0122-png.github.io/ustam/) を参照してください。

## License
[MIT License](LICENSE)
