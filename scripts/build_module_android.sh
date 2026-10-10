#!/usr/bin/env bash
#
# 构建 CPPlayer 的 **QQ 音乐** JNI 音源模块（Android，多 ABI）。
#
# 依赖：
#   - cargo-ndk      安装：cargo install cargo-ndk
#   - Android NDK    设置 ANDROID_NDK_HOME，或放在 $ANDROID_HOME/ndk/<ver>
#
# 产物：target/cp_qq_module_android.zip
#   应用内「设置 → 音源管理 → 导入模块」直接选这个 zip。
#
# 说明：模块包内布局为
#   manifest.json
#   lib/<abi>/libnetease_qq_music_api.so
# 宿主 `PlatformSupport.resolveEntryPoint` 会按设备 ABI 自动挑选对应 .so。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ABIS=(arm64-v8a armeabi-v7a x86_64)
LIB_NAME="libnetease_qq_music_api.so"
OUT_DIR="target/cp_qq_module_android"
ZIP="target/cp_qq_module_android.zip"

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "错误：未找到 cargo-ndk。请先执行：cargo install cargo-ndk" >&2
  exit 1
fi

if [ -z "${ANDROID_NDK_HOME:-}" ] && [ -z "${ANDROID_NDK_ROOT:-}" ]; then
  echo "提示：未设置 ANDROID_NDK_HOME / ANDROID_NDK_ROOT，cargo-ndk 将自行探测 SDK 下的 NDK。" >&2
fi

# Android 15+ 要求 16KB 页面对齐；缺了它高版本系统会拒绝加载 .so。
export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-z,max-page-size=16384"

echo "==> 编译（${ABIS[*]}）"
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ./target/jniLibs \
  build --release --lib --features jni

echo "==> 打包"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
cp packaging/manifest.json "$OUT_DIR/manifest.json"

for abi in "${ABIS[@]}"; do
  src="$(find "./target/jniLibs/$abi" -name '*.so' 2>/dev/null | head -n1 || true)"
  if [ -z "$src" ] || [ ! -f "$src" ]; then
    echo "错误：$abi 未产出 .so，请检查 NDK 与 cargo-ndk 配置。" >&2
    exit 1
  fi
  mkdir -p "$OUT_DIR/lib/$abi"
  cp "$src" "$OUT_DIR/lib/$abi/$LIB_NAME"
  echo "    $abi <- $src"
done

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
