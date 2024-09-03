#!/bin/bash

set -eu

if [ "$(git status . --porcelain)" != "" ]; then
    echo "Error: Git working directory is not clean. Please commit your changes."
    exit 1
fi

cargo test

git push origin

current_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)

echo "Current version: $current_version"
read -rp "Enter the next version: " next_version

sed -i '' "s/^version = \".*\"/version = \"$next_version\"/" Cargo.toml

echo "Updated Cargo.toml with version $next_version"

git add Cargo.toml
git commit -m "Bump version to $next_version"

git tag -a "v$next_version" -m "Release v$next_version"

git push --tags

open "https://github.com/esamattis/rt/releases/new?tag=v$next_version"
