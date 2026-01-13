# home-mq-bot

最小構成のメッセージキュー試作です。ブローカーのバイナリと共有クライアントプロトコルライブラリで構成されています。

## 構成

- `broker/`: サーバーバイナリ（エントリポイント: `broker/src/main.rs`）
- `mq/`: 共有ライブラリクレート（プロトコル型: `mq/src/lib.rs`、クライアント補助: `mq/src/client.rs`）

## 前提

- Rust 2021 edition

## ビルド

対象クレートのディレクトリで実行します。

```sh
cargo build
```

## 実行

ブローカーを起動します（`127.0.0.1:5555` にバインド）。

```sh
cargo run
```

## テスト

```sh
cargo test
```

## 開発補助

```sh
cargo fmt
cargo clippy
```

## 変更時の注意

- ポートやフレームスキーマを変更する場合は `broker/` と `mq/` を両方更新してください。
- JSON のプロトコルフィールドは `snake_case` を維持してください。
