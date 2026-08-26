# crux-finder

Finds the commit that changed a program's behavior and shows the lines in that commit that caused the change.

## Install

```bash
npm i -g crux-finder
```

Or run without installing:

```bash
npx crux-finder who -c "cargo test --quiet" -f HEAD~20..HEAD
```

The package ships a small launcher. On first run it downloads the platform binary from [GitHub Releases](https://github.com/Emran-goat/crux/releases) into `~/.crux/bin` and reuses it afterward. This works even where install scripts are blocked (pnpm, bun defaults).

The binary is also available through cargo, Homebrew, winget, and Scoop — see the [main README](https://github.com/Emran-goat/crux#readme).

## Usage

```text
crux who -c <command> -f <range>
```

- `-c` any command; exit 0 means the behavior holds, anything else means it changed
- `-f` the commit range to search, for example `HEAD~20..HEAD`

Full documentation: https://crux.rweb.site/docs

## License

MIT
