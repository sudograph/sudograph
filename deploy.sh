#!/bin/bash

read -p "Enter new version number:" VERSION

echo -e "npm package: generator-sudograph, prepare and publish"

cd generator-sudograph
sed -E -i "s/(\"version\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
npm publish
cd ..

echo -e "npm package: sudograph, prepare and publish"

cd client
sed -E -i "s/(\"version\": \")(.*)(\")/\1$VERSION\3/" package.json
sed -E -i "s/(\"generator-sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
npm publish
cd ..

echo -e "crate: sudodb, prepare"

cd sudodb
sed -E -i "s/(^version = \")(.*)(\")/\1$VERSION\3/" Cargo.toml
cargo publish --dry-run
cd ..

echo -e "crate: sudograph-generate, prepare"

cd sudograph-generate
sed -E -i "s/(^version = \")(.*)(\")/\1$VERSION\3/" Cargo.toml
cargo publish --dry-run
cd ..

echo -e "crate: sudograph, prepare"

sed -E -i "s/(^version = \")(.*)(\")/\1$VERSION\3/" Cargo.toml
sed -E -i "s/(^sudodb = \{ version = \")(.*)(\", path = \"\.\/sudodb\" \})/\1$VERSION\3/" Cargo.toml
sed -E -i "s/(^sudograph-generate = \{ version = \")(.*)(\", path = \"\.\/sudograph-generate\" \})/\1$VERSION\3/" Cargo.toml
cargo publish --dry-run

echo -e "commit and push final changes"

git add --all
git commit -am "updating to version $VERSION"
git push origin main

echo -e "create and push git tag"

git tag v$VERSION
git push origin v$VERSION

echo -e "crate: sudodb, publish"

cd sudodb
cargo publish
cd ..

echo -e "crate: sudograph-generate, publish"

cd sudograph-generate
cargo publish
cd ..

echo -e "sleeping for 30 seconds before final publish to ensure sudodb and sudograph-generate crates are fully registered on crates.io"

sleep 30

echo -e "crate: sudograph, publish"

cargo publish

echo -e "All packages and crates have been published to version $VERSION"