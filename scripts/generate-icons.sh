#!/bin/bash

# Regenerates all app icons from assets/icon/icon.svg
# Usage: ./scripts/generate-icons.sh
# Requires: rsvg-convert (librsvg) and magick (ImageMagick)

set -e

cd "$(dirname "$0")/.."

SRC="assets/icon/icon.svg"
OUT="assets/icon"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# Full-bleed variant without the macOS safe-area margin, for small sizes
sed 's/viewBox="0 0 1024 1024"/viewBox="100 100 824 824"/' "$SRC" > "$TMP/full.svg"

# Window icon and Linux desktop icon
rsvg-convert -w 256 -h 256 "$TMP/full.svg" -o "$OUT/icon-256.png"
cp "$TMP/full.svg" assets/linux/commit-rewriter.svg

# Windows .ico with the usual sizes
for size in 16 24 32 48 64 128 256; do
    rsvg-convert -w $size -h $size "$TMP/full.svg" -o "$TMP/ico-$size.png"
done
# ICO with PNG-compressed entries (supported since Windows Vista), much
# smaller than the uncompressed bitmaps ImageMagick writes
python3 - "$OUT/icon.ico" "$TMP"/ico-{16,24,32,48,64,128,256}.png <<'PY'
import struct, sys
out, pngs = sys.argv[1], sys.argv[2:]
images = [open(p, "rb").read() for p in pngs]
header = struct.pack("<HHH", 0, 1, len(images))
entries, offset = b"", 6 + 16 * len(images)
for data in images:
    width, height = struct.unpack(">II", data[16:24])
    entries += struct.pack("<BBBBHHII", width % 256, height % 256, 0, 0, 1, 32, len(data), offset)
    offset += len(data)
with open(out, "wb") as f:
    f.write(header + entries + b"".join(images))
PY

# macOS keeps the margin; the .icns is built from this file on macOS
rsvg-convert -w 1024 -h 1024 "$SRC" -o "$OUT/icon-macos-1024.png"

echo "Icons generated in $OUT"
