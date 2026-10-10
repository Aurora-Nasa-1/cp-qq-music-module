#!/usr/bin/env bash
#
# 构建 CPPlayer 的 **QQ 音乐** JNI 音源模块（Linux / macOS 桌面端）。
#
# Windows 请用 scripts/build_module_desktop.ps1。
#
# 产物：target/cp_qq_module_<os>_<arch>.zip
#   应用内「设置 → 音源管理 → 导入模块」直接选这个 zip。
#
# 说明：桌面端为**单架构**，模块包布局为
#   manifest.json
#   <libnetease_qq_music_api.so | .dylib>
# 即库文件放在包根目录（无 lib/<abi>/ 子目录），并去掉 manifest 的 supportedAbis。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

case "$(uname -s)" in
  Linux)
    LIB_NAME="libnetease_qq_music_api.so"
    TAG="linux_$(uname -m)"
    ;;
  Darwin)
    LIB_NAME="libnetease_qq_music_api.dylib"
    TAG="macos_$(uname -m)"
    ;;
  *)
    echo "错误：本脚本只支持 Linux / macOS；Windows 请用 scripts/build_module_desktop.ps1" >&2
    exit 1
    ;;
esac

OUT_DIR="target/cp_qq_module_${TAG}"
ZIP="target/cp_qq_module_${TAG}.zip"

echo "==> 编译（$TAG）"
cargo build --release --lib --features jni

echo "==> 打包"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
cp "target/release/$LIB_NAME" "$OUT_DIR/"

python3 - "$OUT_DIR/manifest.json" "$LIB_NAME" <<'PY'
import json, sys
dst, lib = sys.argv[1], sys.argv[2]
with open("packaging/manifest.json", encoding="utf-8") as f:
    manifest = json.load(f)
manifest["entryPoint"] = lib
manifest.pop("supportedAbis", None)
with open(dst, "w", encoding="utf-8") as f:
    json.dump(manifest, f, ensure_ascii=False, indent=2)
    f.write("\n")
PY

rm -f "$ZIP"
if command -v zip >/dev/null 2>&1; then
  (cd "$OUT_DIR" && zip -qr "$ROOT/$ZIP" .)
else
  python3 - "$OUT_DIR" "$ZIP" <<'PY'
import os, sys, zipfile
src, dst = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(dst, "w", zipfile.ZIP_DEFLATED) as z:
    for root, _, files in os.walk(src):
        for name in files:
            path = os.path.join(root, name)
            z.write(path, os.path.relpath(path, src))
PY
fi

echo "==> 完成：$ZIP"
