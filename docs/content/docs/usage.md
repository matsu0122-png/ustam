---
title: 使い方
weight: 30
description: コマンドラインオプションと実行例
---

基本形式は以下です。

```bash
ustam [OPTIONS] [PATH]
```

`PATH` を省略すると、現在のディレクトリを対象にします。

## オプション

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

## 例

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

詳細形式で表示します。

```bash
ustam -l
```

詳細形式で、サイズの大きい順に表示します。

```bash
ustam -sl
```

更新日時順に並び替えます。

```bash
ustam -t
```

複数のオプションを組み合わせることもできます。

```bash
ustam -al .
```

## シェル補完

`--completions` オプションで、シェルの補完スクリプトを生成できます。Homebrewでインストールした場合は自動的に配置されるため、通常は手動で行う必要はありません。

```bash
ustam --completions bash > /usr/local/etc/bash_completion.d/ustam
ustam --completions zsh  > /usr/local/share/zsh/site-functions/_ustam
```
