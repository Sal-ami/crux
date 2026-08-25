# Packaging

crux installs from more than cargo. Every channel below ships the same binary
from the same GitHub Release. Assets follow `crux-$TARGET.tar.gz` (or `.zip`
on Windows), matching what `install.sh` / `install.ps1` already fetch.

| channel | install command | files here |
|---|---|---|
| cargo | `cargo install crux-finder` (binary is still `crux`) | (crates.io) |
| script | `curl -fsSL https://crux.sh \| sh` / `install.ps1` | `install.sh`, `install.ps1` |
| npm / npx / pnpm / bun / yarn | `npm i -g crux-finder` then `crux`, or just `npx crux-finder` | `npm/` |
| Homebrew | `brew tap Emran-goat/crux <repo>` + `brew install crux` | `homebrew/crux.rb` |
| Scoop | `scoop bucket add crux <repo>` + `scoop install crux` | `scoop/crux.json` |
| winget | `winget install Emran-goat.crux` (after PR to winget-pkgs) | `winget/0.1.0/` |
| Chocolatey | `choco install crux` (after community-repo approval) | `chocolatey/crux/` |
| AUR | `makepkg -si` in `aur/`, or publish as `crux-bin` | `aur/` |
| deb | `cargo deb` (CI on linux) | `Cargo.toml [package.metadata.deb]` |
| rpm | `cargo generate-rpm` (CI on linux) | `Cargo.toml [package.metadata.generate-rpm]` |
| Docker | `docker build -f packaging/Dockerfile -t crux .` | `Dockerfile` |

## Release checklist (per version)

1. Tag `vX.Y.Z` and push. CI builds the 5 targets and uploads:
   - `crux-x86_64-pc-windows-msvc.zip`
   - `crux-x86_64-unknown-linux-musl.tar.gz`
   - `crux-aarch64-unknown-linux-musl.tar.gz`
   - `crux-x86_64-apple-darwin.tar.gz`
   - `crux-aarch64-apple-darwin.tar.gz`
   - each with a `.sha256` sidecar
2. Bump every `version` field in `packaging/npm/*/package.json`,
   `packaging/scoop/crux.json`, `packaging/winget/0.1.0/*`,
   `packaging/chocolatey/crux/crux.nuspec`, `packaging/aur/PKGBUILD` + `.SRCINFO`.
3. Replace every `PLACEHOLDER_SHA256` with real digests
   (`Get-FileHash` / `sha256sum` on the release assets).
4. Publish, in this order:
   - `cd packaging/npm/crux-finder && npm publish`
     then each `packaging/npm/crux-finder-*/` (`npm publish --access public`
     if scoped later)
   - push the Homebrew formula + Scoop manifest to their tap/bucket repos
   - open the winget-pkgs PR with the three manifests
   - `cd packaging/chocolatey/crux && choco pack` then `choco push`
   - AUR: `makepkg --printsrcinfo > .SRCINFO`, commit, `git push` to AUR
   - crates.io: `cargo publish` (needs the repo README.md to exist)
   - deb/rpm: build in the linux CI job, attach to the release
5. Smoke-test after publishing:
   - `npx crux-finder@latest --help`
   - `npm i -g crux-finder && crux --version`
   - `pnpm add -g crux-finder && crux --version`
   - `bunx crux-finder --help`
   - `scoop install crux`, `brew install crux`, `winget install Emran-goat.crux`

## Why the npm package downloads at runtime

pnpm and bun block postinstall scripts by default, and shipping 5 binaries in
5 npm packages means publishing 6 artifacts per release. Instead the main
package ships a tiny shim: it looks for the binary next to the platform
package, then in `~/.crux/bin`, and downloads the release asset on first run
if neither exists. Works everywhere, one source of truth for binaries.

## npm name note

`crux` and `crux-cli` are taken on npm, so the package is `crux-finder` and
the binary it installs is still `crux`.
