#!/usr/bin/env bash
set -euo pipefail

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
changelog="$root/CHANGELOG.md"

[ -f "$changelog" ] || die "no changelog at $changelog"

notes=$(awk -v head="## [$tag]" '
	index($0, head) == 1 { inside = 1; next }
	inside && ($0 == "- - -" || index($0, "## [") == 1) { exit }
	inside { print }
' "$changelog")

[ -n "$notes" ] || die "the changelog holds no section for $tag"

printf '%s\n' "$notes"
