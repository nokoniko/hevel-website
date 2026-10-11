#!/bin/sh
# Builds the static site (pre-rendered with SSG) into ./dist, ready to deploy to Vercel.
set -e
rm -rf target/dx/hevel_rust/release/web dist
dx build --release --web --ssg
cp -R target/dx/hevel_rust/release/web/public dist
cp vercel.json dist/
# Vercel serves /404.html (with a real 404 status) for every address that doesn't exist.
cp dist/error/404/index.html dist/404.html
echo "Static site ready in ./dist"
