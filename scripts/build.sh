#!/bin/bash
set -e

cargo build-sbf --manifest-path=Cargo.toml --sbf-out-dir=dist/program
