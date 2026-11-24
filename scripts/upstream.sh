#!/usr/bin/env bash
# The patched fulgur and Blitz checkouts the build expects next to this
# repository (../fulgur, ../blitz), pinned in .github/upstream.env.
#
#   scripts/upstream.sh fetch   check out the pinned commits into ../fulgur
#                               and ../blitz (shallow; used by CI)
#   scripts/upstream.sh pin     record the current HEADs of ../fulgur and
#                               ../blitz, which must already be pushed
set -euo pipefail
cd "$(dirname "$0")/.."
env_file=.github/upstream.env
# shellcheck source=../.github/upstream.env
. "$env_file"

fetch() { # dir repo rev
  local dir=$1 repo=$2 rev=$3
  if [[ -d $dir/.git ]] && [[ $(git -C "$dir" rev-parse HEAD) == "$rev" ]]; then
    echo "$dir: already at $rev"
    return
  fi
  if [[ -d $dir/.git ]] && [[ -n $(git -C "$dir" status --porcelain) ]]; then
    echo "$dir has local changes; not touching it" >&2
    exit 1
  fi
  [[ -d $dir/.git ]] || git init -q "$dir"
  git -C "$dir" fetch -q --depth 1 "https://github.com/$repo" "$rev"
  git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD
  echo "$dir: $repo at $rev"
}

pin() { # dir name
  local dir=$1 name=$2 rev
  rev=$(git -C "$dir" rev-parse HEAD)
  if [[ -n $(git -C "$dir" status --porcelain --untracked-files=no) ]]; then
    echo "$dir has uncommitted changes" >&2
    exit 1
  fi
  if [[ -z $(git -C "$dir" branch -r --contains "$rev") ]]; then
    echo "$dir: $rev is not pushed to any remote" >&2
    exit 1
  fi
  sed -i.bak "s/^${name}_REV=.*/${name}_REV=$rev/" "$env_file" && rm -f "$env_file.bak"
  echo "${name}_REV=$rev"
}

case ${1:-} in
  fetch)
    fetch ../fulgur "$FULGUR_REPO" "$FULGUR_REV"
    fetch ../blitz "$BLITZ_REPO" "$BLITZ_REV"
    ;;
  pin)
    pin ../fulgur FULGUR
    pin ../blitz BLITZ
    ;;
  *)
    sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
    ;;
esac
