#!/bin/bash

set -eu

if [ "$(git status . --porcelain)" != "" ]; then
    echo "Error: Git working directory is not clean. Please commit your changes."
    exit 1
fi

current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "master" ]; then
    echo "Error: Not on master branch. Please switch to master."
    exit 1
fi

cargo test

git push origin

current_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)

echo "Current version: $current_version"
read -rp "Enter the next version: " next_version

sed -i '' "s/^version = \".*\"/version = \"$next_version\"/" Cargo.toml

echo "Updated Cargo.toml with version $next_version"

cargo build --release

./target/release/rt --version

git add Cargo.toml Cargo.lock
git commit -m "Release v$next_version"

git tag -a "v$next_version" -m "Release v$next_version"

git push
git push --tags

open "https://github.com/esamattis/rt/releases/new?tag=v$next_version"
