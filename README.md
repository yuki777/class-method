# class-method

PHP ファイルのクラス・メソッド行数を計測し、行数の多い順にランキング表示する CLI ツール。

[tree-sitter-php](https://github.com/tree-sitter/tree-sitter-php) による高速な構文解析と [rayon](https://github.com/rayon-rs/rayon) による並列処理で、数百ファイルを一瞬で解析します。Bash + PHPMD 版と比較して約 **500倍高速** (21.4秒 → 0.043秒, 883ファイル)。

## インストール

### GitHub Releases からダウンロード

[Releases](https://github.com/yuki777/class-method/releases) から対象プラットフォームのバイナリをダウンロードして展開:

```bash
curl -sL https://github.com/yuki777/class-method/releases/latest/download/class-method-darwin-arm64.tar.gz | tar xz
mv class-method-darwin-arm64 /usr/local/bin/class-method
```

### ソースからビルド

```bash
git clone https://github.com/yuki777/class-method.git
cd class-method
cargo build --release
# バイナリは target/release/class-method に生成されます
```

## 使い方

```
class-method [OPTIONS] [TARGET]
```

### オプション

| オプション | デフォルト | 説明 |
|-----------|-----------|------|
| `-n NUM` | 10 | 表示件数 |
| `-t TYPE` | both | `method` / `class` / `both` |
| `-v, --version` | | バージョン表示 |
| `-h, --help` | | ヘルプ表示 |
| `TARGET` | src | 対象ディレクトリ |

### 例

```bash
# src 配下の Top10（デフォルト）
class-method

# メソッドのみ Top20
class-method -n 20 -t method

# 特定ディレクトリに絞る
class-method -n 5 src/Resource

# クラスのみ
class-method -t class
```

### 出力例

```
=== ExcessiveClassLength (クラス行数) Top 5 ===

Lines   File                                                  Name
-----   ----                                                  ----
1432    src/Resource/App/Migration/Article.php:53              Article
1251    src/Resource/App/Migration/Blog.php:50                 Blog
634     src/Resource/App/Migration/Article/Body.php:40         Body
624     src/Resource/App/Migration/Magazine.php:31             Magazine
573     src/Service/Rss/RssArticleBody.php:35                  RssArticleBody

=== ExcessiveMethodLength (メソッド行数) Top 5 ===

Lines   File                                                  Name
-----   ----                                                  ----
350     src/Module/AppModule.php:208                           configure
187     src/Resource/App/Migration/Article/Body.php:306        createBlockRequest
181     src/Provider/QiqHelperLocatorProvider.php:82            get
169     src/Service/Rss/RssArticleBody.php:240                 applySmartnewsBodyLinkExclusions
153     src/Resource/App/Migration/Article/Body/Related.php:160 buildSearch
```

## 対象とする PHP 構文

- class
- trait
- interface
- enum
- anonymous class
- method (上記の内部で宣言されたもの)

## 行数の数え方

宣言の開始行（PHP 8 アトリビュートを除く）から閉じ括弧 `}` の行までを1単位として数えます。メソッド前の PHPDoc コメントは含みません。

## License

MIT
