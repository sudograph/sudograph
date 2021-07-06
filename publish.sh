#!/bin/bash

set -e

read -p "Enter new version number:" VERSION

echo -e "npm package: generator-sudograph, prepare and publish"

cd generator-sudograph
sed -E -i "s/(\"version\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install

cd generators/app/templates/canisters

cd frontend
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
cd ..

cd graphql
sed -E -i "s/(^sudograph = \")(.*)(\")/\1$VERSION\3/" Cargo.toml
cd ..

cd playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
cd ../../../../..

if [[ "$VERSION" == *"-beta."* ]];
then
    npm publish --tag beta
else
    npm publish
fi

# TODO instead of sleeping we could probably poll in a loop using some kind of npm API check
echo -e "sleeping for 30 seconds to ensure generator-sudograph is fully registered on npm"

sleep 30

cd ..

echo -e "npm package: sudograph, prepare and publish"

cd client
sed -E -i "s/(\"version\": \")(.*)(\")/\1$VERSION\3/" package.json
sed -E -i "s/(\"generator-sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install

if [[ "$VERSION" == *"-beta."* ]];
then
    npm publish --tag beta
else
    npm publish
fi

cd ..

echo -e "example: basic"

cd examples/basic/canisters/graphql
sed -E -i "s/(^sudograph = \{ version = \")(.*)(\", path = \"\.\.\/\.\.\/\.\.\/\.\.\/\.\.\/sudograph\" \})/\1$VERSION\3/" Cargo.toml
cd ../frontend
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../../../..

echo -e "example: files"

cd examples/files/canisters/graphql
sed -E -i "s/(^sudograph = \{ version = \")(.*)(\", path = \"\.\.\/\.\.\/\.\.\/\.\.\/\.\.\/sudograph\" \})/\1$VERSION\3/" Cargo.toml
cd ../files
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../../../..

echo -e "example: intermediate"

cd examples/intermediate/canisters/graphql
sed -E -i "s/(^sudograph = \{ version = \")(.*)(\", path = \"\.\.\/\.\.\/\.\.\/\.\.\/\.\.\/sudograph\" \})/\1$VERSION\3/" Cargo.toml
cd ../playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../../../..

echo -e "example: motoko-client"

cd examples/motoko-client/canisters/graphql
sed -E -i "s/(^sudograph = \{ version = \")(.*)(\", path = \"\.\.\/\.\.\/\.\.\/\.\.\/\.\.\/sudograph\" \})/\1$VERSION\3/" Cargo.toml
cd ../playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../../../..

echo -e "example: motoko-custom-resolvers"

cd examples/motoko-custom-resolvers/canisters/graphql
sed -E -i "s/(^sudograph = \{ version = \")(.*)(\", path = \"\.\.\/\.\.\/\.\.\/\.\.\/\.\.\/sudograph\" \})/\1$VERSION\3/" Cargo.toml
cd ../playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../../../..

echo -e "example: rust-client"

cd examples/rust-client/canisters/graphql
sed -E -i "s/(^sudograph = \{ version = \")(.*)(\", path = \"\.\.\/\.\.\/\.\.\/\.\.\/\.\.\/sudograph\" \})/\1$VERSION\3/" Cargo.toml
cd ../playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../../../..

echo -e "example: rust-custom-resolvers"

cd examples/rust-custom-resolvers/canisters/graphql
sed -E -i "s/(^sudograph = \{ version = \")(.*)(\", path = \"\.\.\/\.\.\/\.\.\/\.\.\/\.\.\/sudograph\" \})/\1$VERSION\3/" Cargo.toml
cd ../playground
sed -E -i "s/(\"sudograph\": \")(.*)(\")/\1$VERSION\3/" package.json
npm install
cd ../../../..

echo -e "canister: The Sudograph Book"

npm run book-build
cd sudograph-book
dfx deploy --network ic
cd ..

echo -e "crate: sudodb, prepare"

cd sudodb
sed -E -i "s/(^version = \")(.*)(\")/\1$VERSION\3/" Cargo.toml
cargo build
cd ..

echo -e "crate: sudograph-generate, prepare"

cd sudograph-generate
sed -E -i "s/(^version = \")(.*)(\")/\1$VERSION\3/" Cargo.toml
cargo build
cd ..

echo -e "crate: sudograph, prepare"

sed -E -i "s/(^version = \")(.*)(\")/\1$VERSION\3/" Cargo.toml
sed -E -i "s/(^sudodb = \{ version = \")(.*)(\", path = \"\.\/sudodb\" \})/\1$VERSION\3/" Cargo.toml
sed -E -i "s/(^sudograph-generate = \{ version = \")(.*)(\", path = \"\.\/sudograph-generate\" \})/\1$VERSION\3/" Cargo.toml
cargo build

echo -e "commit and push final changes"

git add --all
git commit -am "updating to version $VERSION"
git push origin main

echo -e "create and push git tag"

git tag v$VERSION
git push origin v$VERSION

echo -e "crate: sudodb, publish"

cd sudodb
cargo publish --dry-run
cargo publish
cd ..

echo -e "crate: sudograph-generate, publish"

cd sudograph-generate
cargo publish --dry-run
cargo publish
cd ..

# TODO instead of sleeping we could probably poll in a loop using some kind of crates.io API check
echo -e "sleeping for 30 seconds before final publish to ensure sudodb and sudograph-generate crates are fully registered on crates.io"

sleep 30

echo -e "crate: sudograph, publish"

cargo publish --dry-run
cargo publish

echo -e "All packages, crates, and canisters have been published to version $VERSION"