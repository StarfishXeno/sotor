#!/bin/bash
set -e
set -o allexport
source .env
set +o allexport

name="sotor"
targets=("x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu")
exts=(".exe" "")
suffix=("win" "linux")

rm -rf ./target/build
mkdir -p ./target/build

clean() {
    rm -rf ./target/release;
}

for i in "${!targets[@]}"; do
    target=${targets[i]}
    ext=${exts[i]}
    suffix=${suffix[i]}

    # artifacts from previous builds mess cross up
    clean
    echo Building for $target;
    cross build --release --target $target;
    mv ./target/$target/release/$name$ext ./target/build/$name-$suffix$ext;
    echo Done with $target;
done

clean
echo Building for web;
trunk build --release   
echo Done with web;
