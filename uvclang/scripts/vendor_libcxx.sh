#!/usr/bin/env bash
set -euo pipefail

# Vendors a pinned copy of libc++ (include/ + src/ for reference) into
# uvclang/third_party/libcxx/. Re-run with a different LIBCXX_TAG to update
# the pin. Requires network access to github.com/llvm/llvm-project.
#
# LIBCXX_TAG should track the major version of whatever clang++ uvclang's find_clang() resolves to
# on this machine 

LIBCXX_TAG="${LIBCXX_TAG:-llvmorg-22.1.8}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UVCLANG_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST="$UVCLANG_DIR/third_party/libcxx"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "Fetching llvm/llvm-project @ ${LIBCXX_TAG} (sparse: libcxx/) ..."
git clone --depth 1 --branch "$LIBCXX_TAG" --filter=blob:none --sparse \
  https://github.com/llvm/llvm-project.git "$WORK/llvm-project"
git -C "$WORK/llvm-project" sparse-checkout set libcxx

rm -rf "$DEST"
mkdir -p "$DEST"
cp -R "$WORK/llvm-project/libcxx/include" "$DEST/include"
cp -R "$WORK/llvm-project/libcxx/src" "$DEST/src"
cp "$WORK/llvm-project/libcxx/LICENSE.TXT" "$DEST/LICENSE.TXT"

find "$DEST" -name '.git*' -prune -exec rm -rf {} +

cat > "$DEST/VERSION" <<EOF
Vendored from llvm/llvm-project @ ${LIBCXX_TAG}
Fetched: $(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF

echo "Vendored libc++ (${LIBCXX_TAG}) into ${DEST}"
