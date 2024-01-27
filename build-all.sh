#!/bin/bash
set -e
set -o allexport
source .env
set +o allexport

name="sotor"
targets=("x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu")
exts=(".exe" "")
suffix=("win" "linux")

mkdir -p ./target/build

for i in "${!targets[@]}"; do
    target=${targets[i]}
    ext=${exts[i]}
    suffix=${suffix[i]}

    # artifacts from previous builds mess cross up
    rm -rf ./target/release;
    echo Building $target;
    cross build --release --target $target;
    mv ./target/$target/release/$name$ext ./target/build/$name-$suffix$ext;
    echo Done with $target;
done