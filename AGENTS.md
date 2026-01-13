# Repository Guidelines

このリポジトリは、ブローカーのバイナリと共有クライアントプロトコルライブラリからなる最小構成のメッセージキュー試作です。変更内容を揃えてレビューしやすくするため、以下の方針に従ってください。やりとりは日本語で行ってください。

## Project Structure & Module Organization

- `broker/` はサーバーバイナリです。エントリポイントは `broker/src/main.rs`。
- `mq/` は共有ライブラリクレートです。プロトコル型は `mq/src/lib.rs`、クライアント補助は `mq/src/client.rs`。
- まだ `tests/` ディレクトリはありません。テストは `tests/` か `#[cfg(test)]` のモジュール内に置けます。

## Build, Test, and Development Commands

対象クレートのディレクトリで実行します。

- `cargo build`（`broker/` または `mq/`）：ビルド。
- `cargo run`（`broker/`）：`127.0.0.1:5555` でブローカー起動。
- `cargo test`（`broker/` または `mq/`）：ユニット/結合テスト実行。
- `cargo fmt` と `cargo clippy`（各クレート）：整形と静的解析。

## Coding Style & Naming Conventions

- Rust 2021 edition。`rustfmt` のデフォルト（4スペース、末尾カンマ）に従う。
- 命名は `snake_case`（関数/変数/モジュール）、`CamelCase`（型/列挙）、`SCREAMING_SNAKE_CASE`（定数）。
- JSON のプロトコルフィールドは `snake_case` を維持（`mq::Frame` 参照）。
- `mq/` は小さく焦点を絞ったヘルパー、ブローカーのロジックは `broker/` に集約。

## Testing Guidelines

- 既存のテストはありません。追加する場合は `mod tests` か `tests/` に配置。
- テスト名は `test_*`。シリアライズ、pub/sub フロー、エラーケースを優先。
- 非同期テストは `#[tokio::test]` を使用。

## Commit & Pull Request Guidelines

- この環境では Git 履歴を参照できないため、コミットは命令形で簡潔に（例: "Add pubsub delivery ack"）。
- PR には概要、実行/検証手順、プロトコルやポート変更の有無を記載。

## Configuration & Security Tips

- ブローカーは既定で `127.0.0.1:5555` にバインド。認証やアクセス制御を追加するまではローカルのままにする。
- ポートやフレームスキーマを変更する場合は `broker/` と `mq/` を両方更新し、PR に明記。
