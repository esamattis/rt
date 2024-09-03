#!/bin/sh

set -eu

exec cargo install --debug --path . --offline
