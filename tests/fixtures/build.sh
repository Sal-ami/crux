#!/bin/sh
# Build fixture repos for integration tests
set -e

# simple: basic behavior flip
cd tests/fixtures/simple
rm -rf .git
git init
git config user.name "test"
git config user.email "test@test.com"
echo "pass" > file.txt
git add . && git commit -m "good"
echo "fail" > file.txt
git add . && git commit -m "bad"
echo "fail" > file.txt
git add . && git commit -m "also bad"
cd ../../..

# interaction: two commits that break together
cd tests/fixtures/interaction
rm -rf .git
git init
git config user.name "test"
git config user.email "test@test.com"
echo "pass" > file.txt
git add . && git commit -m "base"
echo -e "pass\nextra" > file.txt
git add . && git commit -m "add extra"
echo -e "#!/bin/sh\ngrep -q pass file.txt && grep -q extra file.txt" > test.sh
chmod +x test.sh
git add . && git commit -m "change test"
cd ../../..

# dep_chain: vendored dependency
cd tests/fixtures/dep_chain
rm -rf .git
git init
git config user.name "test"
git config user.email "test@test.com"
mkdir -p vendor/lib
echo "fn compute(x: i32) -> i32 { x }" > vendor/lib/mod.rs
echo 'name = "lib"' > vendor/lib/Cargo.toml
echo -e '[package]\nname = "app"\nversion = "0.1.0"' > Cargo.toml
echo 'fn main() { println!("{}", lib::compute(1)); }' > src/main.rs
mkdir -p src
echo 'fn main() { println!("{}", lib::compute(1)); }' > src/main.rs
git add . && git commit -m "initial"
echo "fn compute(x: i32) -> i32 { x + 1 }" > vendor/lib/mod.rs
git add . && git commit -m "update lib"
echo "fn compute(x: i32) -> i32 { x * 2 }" > vendor/lib/mod.rs
git add . && git commit -m "update lib again"
cd ../../..
