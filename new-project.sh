#!/usr/bin/env bash
# Creates a new app from template/.
#
#   ./new-project.sh <app-id> "<Title>" <target-dir>
#   ./new-project.sh my-app "My App" ~/projects/my-app
#
# <app-id>: lowercase letters, digits and "-"; also the binary, crate and .desktop name.
# KAAMOS_PATH=<this repo> uses path dependencies instead of the git tag (for testing).
set -euo pipefail

if [ $# -ne 3 ]; then
    sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
fi
app=$1
title=$2
dest=$3

if ! [[ $app =~ ^[a-z][a-z0-9-]*$ ]]; then
    echo "app id must be lowercase letters, digits and -: $app" >&2
    exit 1
fi
if [[ $title == *'"'* || $title == *\\* ]]; then
    echo "title must not contain \" or \\ (it goes into Cargo.toml, Rust and Slint strings)" >&2
    exit 1
fi
if [ -e "$dest" ]; then
    echo "already exists: $dest" >&2
    exit 1
fi

here=$(cd "$(dirname "$0")" && pwd)
cp -r "$here/template" "$dest"
mv "$dest/gitignore" "$dest/.gitignore"
mv "$dest/packaging/__APP__.desktop" "$dest/packaging/$app.desktop"
mv "$dest/packaging/__APP__.svg" "$dest/packaging/$app.svg"

# Escape the title for sed (\, & and the | delimiter).
title_sed=$(printf '%s' "$title" | sed 's/[\\&|]/\\&/g')
grep -rlI -e "__APP__" -e "__TITLE__" "$dest" | while read -r f; do
    sed -i "s|__APP__|$app|g; s|__TITLE__|$title_sed|g" "$f"
done

if [ -n "${KAAMOS_PATH:-}" ]; then
    kit=$(cd "$KAAMOS_PATH" && pwd)
    sed -i "s|^kaamos = { git = .*|kaamos = { path = \"$kit/crates/kaamos\" }|; \
            s|^kaamos-ui = { git = .*|kaamos-ui = { path = \"$kit/crates/ui\" }|" "$dest/Cargo.toml"
fi

git -C "$dest" init -q -b main
echo "Created $dest ($app, \"$title\"). Next: cd $dest && ./check.sh && cargo run"
