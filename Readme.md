# nanai-gna-dll-rs

RustでIntel GNA (Gaussian & Neural Accelerator) のランタイムDLL/共有ライブラリを動的ロード（libloading）して使用するためのライブラリです。

## 特長

- **ビルド時リンク不要**: 実行時に動的ロードするため、開発環境に事前に GNA の lib/dll を配置・リンクする必要がありません。
- **柔軟なロードインターフェイス**:
  - 任意のファイルパスからのロード (`load_from_path`)
  - 任意の環境変数からのロード (`load_from_env`)
  - 環境変数優先＋フォールバック指定 (`load_from_env_or_path`)
  - ビルダーパターンによる柔軟な探索設定 (`GnaLibrary::builder()`)
- **安全なRAII管理**:
  - `GnaLibrary` (DLLハンドル)
  - `GnaDevice` (デバイスオープン/クローズ)
  - `GnaBuffer` (GNAアロケータメモリの確保と自動解放)

## 使い方

### 1. 任意のファイルパスからロード

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::load_from_path("C:/path/to/gna.dll")?;
```

### 2. 任意の環境変数からロード

指定した環境変数にDLLのファイルパス、またはDLLが配置されているディレクトリパスを設定してロードできます（ディレクトリの場合は `gna.dll` / `libgna.so` が自動補完されます）。

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::load_from_env("CUSTOM_GNA_LIB_PATH")?;
```

### 3. 環境変数またはフォールバックパス

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::load_from_env_or_path("CUSTOM_GNA_LIB_PATH", "fallback/gna.dll")?;
```

### 4. ビルダーによる優先度付き探索

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::builder()
    .path("explicit/path/to/gna.dll")      // 1. 指定パス
    .env("MY_APP_GNA_DIR")                 // 2. 指定した環境変数
    .default_envs()                        // 3. 標準の GNA_LIB_PATH, GNA_LIB_DIR
    .current_dir()                         // 4. カレントディレクトリ
    .system_fallback(true)                 // 5. システム検索パス
    .load()?;
```

### 5. デフォルト検索ルールでのロード

```rust
use nanai_gna_dll_rs::GnaLibrary;

// GNA_LIB_PATH -> GNA_LIB_DIR -> カレントディレクトリ -> システムパス の順で探索
let lib = GnaLibrary::load_default()?;
```

## CLIデモ

```bash
# ヘルプ表示
cargo run -- --help

# 任意のパスを指定して実行
cargo run -- --dll path/to/gna.dll

# 任意の環境変数を指定して実行
cargo run -- --env MY_GNA_LIB_PATH

# デフォルト探索
cargo run
```
