---
title: ustam
---

{{% blocks/cover title="ustam" height="med" color="primary" %}}
Rustで実装した拡張版 `ls` コマンド風CLIツール

<div class="mx-auto">
  <a class="btn btn-lg btn-light me-3 mb-4" href="{{< relref "/docs" >}}">ドキュメントを読む</a>
  <a class="btn btn-lg btn-outline-light me-3 mb-4" href="https://github.com/matsu0122-png/ustam">GitHub</a>
</div>
{{% /blocks/cover %}}

{{% blocks/lead %}}
指定したディレクトリのファイル・ディレクトリ一覧を、詳細表示・並び替え・`.gitignore`を考慮した除外付きで表示するCLIツールです。
{{% /blocks/lead %}}

{{% blocks/section color="white" type="row" %}}
{{% blocks/feature icon="fa-terminal" title="拡張された ls" %}}
種別・サイズ・更新日時に加え、READMEのTaglineやPDFのタイトルまで一覧に表示します。
{{% /blocks/feature %}}

{{% blocks/feature icon="fa-download" title="複数チャネルで配布" %}}
Homebrew、Docker（ghcr.io）、GitHub Releasesのバイナリから、環境に合わせてインストールできます。
{{% /blocks/feature %}}

{{% blocks/feature icon="fa-keyboard" title="シェル補完" %}}
bash・zsh・fish・PowerShell・elvish向けの補完スクリプトを自動生成できます。
{{% /blocks/feature %}}
{{% /blocks/section %}}
