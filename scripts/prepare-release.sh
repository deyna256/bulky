#!/usr/bin/env bash
set -euo pipefail

version=$(cargo metadata --no-deps --locked --format-version 1 | jq -er '.packages[] | select(.name == "bulky") | .version')
tag="v$version"
sha=$(git rev-parse HEAD)

if [[ "$GITHUB_EVENT_NAME" == workflow_dispatch ]]; then
  if [[ "$GITHUB_REF" != refs/heads/main ]]; then
    echo "::error::Run Release from main."
    exit 1
  fi
elif [[ "$GITHUB_REF" != "refs/tags/$tag" ]]; then
  echo "::error::Expected tag $tag for this Cargo.toml."
  exit 1
fi

index=$(curl --fail --silent --show-error --retry 3 https://index.crates.io/bu/lk/bulky)
checksum=$(jq -sr --arg v "$version" 'map(select(.vers == $v)) | first | .cksum // empty' <<< "$index")
published=false
if [[ -n "$checksum" ]]; then
  archive="$RUNNER_TEMP/bulky-$version.crate"
  curl --fail --silent --show-error --retry 3 "https://static.crates.io/crates/bulky/bulky-$version.crate" -o "$archive"
  echo "$checksum  $archive" | sha256sum --check
  source_sha=$(tar -xzOf "$archive" "bulky-$version/.cargo_vcs_info.json" |
    jq -er 'select(.git.dirty != true) | .git.sha1 | select(test("^[a-f0-9]{40}$"))')
  if [[ "$GITHUB_EVENT_NAME" == release && "$source_sha" != "$sha" ]]; then
    echo "::error::This release tag differs from the commit published on crates.io."
    exit 1
  fi
  sha=$source_sha
  published=true
fi

git merge-base --is-ancestor "$sha" origin/main
if [[ "$published" == true ]]; then
  git checkout --quiet --detach "$sha"
  cargo package --locked
  echo "$checksum  target/package/bulky-$version.crate" | sha256sum --check
fi

if git show-ref --verify --quiet "refs/tags/$tag" && [[ "$(git rev-parse "refs/tags/$tag^{commit}")" != "$sha" ]]; then
  echo "::error::Tag $tag already points to a different commit."
  exit 1
fi

printf 'tag=%s\nsha=%s\npublished=%s\n' "$tag" "$sha" "$published" >> "$GITHUB_OUTPUT"
echo "Release $tag at $sha (already on crates.io: $published)."
