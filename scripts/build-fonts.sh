#!/bin/sh
# Builds the static Latin TTFs in assets/fonts/ that are compiled into the
# binary. Typst picks faces out of its font book by family, weight and style,
# so all four keep "Noto Serif" as their family name and differ only by weight
# class and italic bit.
#
# The sources are the googlefonts/variable-ttf faces from the Noto Serif 2.015
# release of notofonts/latin-greek-cyrillic, downloaded and checksummed here.
# Set NOTO_SOURCE_DIR to a directory holding them to build offline.
set -eu

cd "$(dirname "$0")/.."

version=2.015
release=https://github.com/notofonts/latin-greek-cyrillic/releases/download/NotoSerif-v$version/NotoSerif-v$version.zip
upright_sha=4d8e6761424656867019081a1a01336f3cb086982682698714054fc33f782713
italic_sha=e9342c2b2debeee282a945e6dffde94612edd7e7b70fba9463abdb6e658ec724

# fontTools stamps each font with the time it was saved unless this is set. A
# fixed date, the 2.015 release date, makes rebuilds byte-identical.
export SOURCE_DATE_EPOCH=1732060800

# Google Fonts' "latin" subset, verbatim.
unicodes='U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD'

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

source_dir=${NOTO_SOURCE_DIR:-$tmp/source}
if [ -z "${NOTO_SOURCE_DIR:-}" ]; then
  mkdir -p "$source_dir"
  echo "downloading Noto Serif $version"
  curl -sSL -o "$tmp/noto.zip" "$release"
  unzip -q -j "$tmp/noto.zip" 'NotoSerif/googlefonts/variable-ttf/*' -d "$source_dir"
fi

upright="$source_dir/NotoSerif[wdth,wght].ttf"
italic="$source_dir/NotoSerif-Italic[wdth,wght].ttf"

verify() {
  echo "$2  $1" | sha256sum --check --status ||
    { echo "checksum mismatch: $1" >&2; exit 1; }
}
verify "$upright" "$upright_sha"
verify "$italic" "$italic_sha"

fonttools() {
  uvx --quiet --from 'fonttools==4.65.0' "$@"
}

# build SOURCE NAME WEIGHT writes assets/fonts/NotoSerif-NAME.ttf. Upstream
# also renames the family after the instanced weight; that would present four
# unrelated families and break weight selection, so it is skipped.
build() {
  fonttools fonttools varLib.instancer "$1" \
    "wght=$3" wdth=100 --quiet \
    --output "$tmp/$2.ttf"
  fonttools pyftsubset "$tmp/$2.ttf" \
    --unicodes="$unicodes" --layout-features='*' --name-IDs='*' \
    --output-file="assets/fonts/NotoSerif-$2.ttf"
  echo "assets/fonts/NotoSerif-$2.ttf"
}

build "$upright" Regular 400
build "$upright" SemiBold 600
build "$upright" ExtraBold 800
build "$italic" Italic 400
