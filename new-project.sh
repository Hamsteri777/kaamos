#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Creates a new app from template/.
#
#   ./new-project.sh <app-id> "<Title>" <target-dir>
#   ./new-project.sh my-app "My App" ~/projects/my-app
#
# <app-id>: lowercase letters, digits and "-"; also the binary, crate and .desktop name.
# <Title>: shown in the window, menu and package; no " \ $ ` or control characters.
# KAAMOS_PATH=<this repo> uses path dependencies instead of the git tag (for testing).
#
# (end of usage)
set -euo pipefail

usage() {
    sed -n '4,/^# (end of usage)/p' "$0" | sed '$d; s/^# \{0,1\}//'
}

if [ $# -ne 3 ]; then
    usage
    exit 2
fi
app=$1
title=$2
dest=$3

# Names cargo refuses for a package or binary: its own directories, the built-in crates and
# Rust keywords.
reserved=" build deps examples incremental test core std alloc proc-macro
    as async await break const continue crate do dyn else enum extern false final fn for gen
    if impl in let loop macro match mod move mut override priv pub ref return self static
    struct super trait true try type typeof unsafe unsized use virtual where while yield
    abstract become box "
if ! [[ $app =~ ^[a-z][a-z0-9-]*$ ]]; then
    echo "app id must be lowercase letters, digits and -, starting with a letter: $app" >&2
    exit 1
fi
if [[ $reserved =~ [[:space:]]$app[[:space:]] ]]; then
    echo "app id is reserved by cargo or Rust: $app" >&2
    exit 1
fi

# The title goes into Cargo.toml, Rust and Slint strings and the PKGBUILD, which makepkg runs
# as a shell script: $ and ` there would execute.
if [ -z "$title" ]; then
    echo "title must not be empty" >&2
    exit 1
fi
if [[ $title == *[\"\\\$\`]* || $title =~ [[:cntrl:]] ]]; then
    echo "title must not contain \" \\ \$ \` or control characters" >&2
    exit 1
fi

if [ -e "$dest" ]; then
    echo "already exists: $dest" >&2
    exit 1
fi
parent=$(dirname -- "$dest")
if [ ! -d "$parent" ]; then
    echo "parent directory does not exist: $parent" >&2
    exit 1
fi

kit=
if [ -n "${KAAMOS_PATH:-}" ]; then
    kit=$(cd -- "$KAAMOS_PATH" && pwd)
    if [[ $kit == *[\"\\\|\&]* ]]; then
        echo "KAAMOS_PATH must not contain \" \\ | or &: $kit" >&2
        exit 1
    fi
fi

here=$(cd -- "$(dirname -- "$0")" && pwd)

# Build next to the target and move it into place at the end, so a failure leaves nothing.
work=$(mktemp -d -- "$parent/.new-project.XXXXXX")
trap 'rm -rf -- "$work"' EXIT
tmp=$work/app

cp -r -- "$here/template" "$tmp"
mv -- "$tmp/gitignore" "$tmp/.gitignore"
mv -- "$tmp/packaging/__APP__.desktop" "$tmp/packaging/$app.desktop"
mv -- "$tmp/packaging/__APP__.svg" "$tmp/packaging/$app.svg"

# Escape the title for sed (\, & and the | delimiter; \ is already refused).
title_sed=$(printf '%s' "$title" | sed 's/[\\&|]/\\&/g')
grep -rlIZ -e "__APP__" -e "__TITLE__" -- "$tmp" | while IFS= read -r -d '' f; do
    sed -i "s|__APP__|$app|g; s|__TITLE__|$title_sed|g" -- "$f"
done

if [ -n "$kit" ]; then
    sed -i "s|^kaamos = { git = .*|kaamos = { path = \"$kit/crates/kaamos\" }|; \
            s|^kaamos-ui = { git = .*|kaamos-ui = { path = \"$kit/crates/ui\" }|" -- "$tmp/Cargo.toml"
fi

git -C "$tmp" init -q -b main
mv -T -- "$tmp" "$dest"
echo "Created $dest ($app, \"$title\"). Next: cd $dest && ./check.sh && cargo run"
