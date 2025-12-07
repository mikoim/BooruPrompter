# BooruPrompter

Booru 系タグの入力を支援する 3 ペイン構成のプロンプトエディタです。Dioxus 0.7 (web) 上で動作し、同梱の Danbooru 辞書を使ってサジェスト・並べ替え・クリップボード操作を提供します。

## 主な特徴
- 3 つのペイン構成: `Prompt Editor` で入力、`Suggestion` でタグ候補を表示、`Tag List` で結果を整列。
- 辞書サジェスト: `assets/danbooru*.csv` を読み込み、前方一致と簡易あいまい検索で候補を提示。カテゴリごとに色分け。
- 並べ替えモード: Original / Custom / A-Z / Fav / Category をワンクリックで切替、改行区切りを保ったままセグメント内をソート。
- 30ms デバウンス入力とカーソル位置追跡で、入力中のトークンだけを安全に置換。
- クリップボード連携と永続化: コピー/貼り付けボタンとローカルストレージへの自動保存（web 時）。
- デスクトップ/モバイルビルドも可能な Dioxus 0.7 構成（`Cargo.toml` の feature で切替）。

## セットアップ
1. Rust 環境を用意します。
2. Dioxus CLI をインストールします（未導入の場合）。
   ```bash
   curl -sSL http://dioxus.dev/install.sh | sh
   ```

## 実行・開発
開発サーバー（web; Tailwind 自動コンパイル込み）:
```bash
dx serve
```
デスクトップで試す場合:
```bash
dx serve --platform desktop
```
本番ビルド例:
```bash
dx build --release --platform web
```

## 使い方
- プロンプト欄にタグをカンマまたは改行区切りで入力すると、候補が `Suggestion` に出ます。クリックまたは Enter で挿入できます。
- ツールバーでクリア/コピー/貼り付け、並べ替えモード変更が可能です。`Tag List` ではタグ単位の移動・削除を行えます（改行行は固定）。
- ソート結果やサジェストは辞書読み込み完了後に反映されます（初回ロード時はステータスバーに進捗を表示）。

## ディレクトリ構成
```
assets/
  danbooru.csv              # カテゴリ情報
  danbooru-machine-jp.csv   # 日本語メタデータ
  styling/main.css          # 手書きスタイル
  tailwind.css              # 自動ビルド対象のベース
src/
  main.rs                   # エントリポイント
  components/               # UI コンポーネント群
  dictionary.rs             # 辞書読込とサジェスト生成
  sort.rs                   # 並べ替えロジック
  state.rs                  # アプリ状態とアクション
  tags.rs                   # タグの抽出・整形
```

## Dioxus 0.7 のエントリ例
```rust
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { components::AppShell {} }
}
```
`dioxus::launch` でランタイムを開始し、`#[component]` で宣言したルートコンポーネントをマウントします。UI は RSX で記述し、`rsx!` ブロック内で要素・コンポーネントを直接組み立てます。

## メモ
- 辞書ファイルは `include_str!` でバンドルしているため、編集後は再ビルドが必要です。
- 自動 Tailwind が有効なため追加設定なしで `dx serve` だけでスタイルが反映されます。カスタマイズする場合は `tailwind.css` を編集してください。
