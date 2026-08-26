#!/usr/bin/env node
"use strict";
const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const VERSION = require("../package.json").version;
const REPO = "Emran-goat/crux";

function platformKey() {
  const { platform, arch } = process;
  if (platform === "win32" && arch === "x64") return "win32-x64";
  if (platform === "linux" && arch === "x64") return "linux-x64-musl";
  if (platform === "linux" && arch === "arm64") return "linux-arm64-musl";
  if (platform === "darwin" && arch === "x64") return "darwin-x64";
  if (platform === "darwin" && arch === "arm64") return "darwin-arm64";
  return null;
}

function targetTriple(key) {
  return {
    "win32-x64": "x86_64-pc-windows-msvc",
    "linux-x64-musl": "x86_64-unknown-linux-musl",
    "linux-arm64-musl": "aarch64-unknown-linux-musl",
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
  }[key];
}

function exeName() {
  return process.platform === "win32" ? "crux.exe" : "crux";
}

function candidatePaths() {
  const key = platformKey();
  const exe = exeName();
  const list = [];
  if (key) {
    const pkg = `crux-finder-${key}`;
    list.push(path.join(__dirname, "..", "..", pkg, exe)); // sibling in node_modules
    list.push(path.join(__dirname, "..", pkg, exe)); // vendored inside package
    list.push(path.join(__dirname, "..", "..", pkg, "bin", exe));
  }
  list.push(path.join(os.homedir(), ".crux", "bin", exe));
  return list;
}

function findBinary() {
  for (const p of candidatePaths()) {
    if (fs.existsSync(p)) return p;
  }
  return null;
}

function download() {
  const key = platformKey();
  if (!key) {
    console.error(`crux: unsupported platform ${process.platform}-${process.arch}`);
    process.exit(1);
  }
  const triple = targetTriple(key);
  const ext = process.platform === "win32" ? "zip" : "tar.gz";
  const asset = `crux-${triple}.${ext}`;
  const urls = [
    `https://github.com/${REPO}/releases/download/v${VERSION}/${asset}`,
    `https://github.com/${REPO}/releases/latest/download/${asset}`,
  ];
  const dest = path.join(os.homedir(), ".crux", "bin");
  fs.mkdirSync(dest, { recursive: true });
  const exe = path.join(dest, exeName());
  const fetchScript = `
    const fs = require("fs"), { execSync } = require("child_process");
    const urls = JSON.parse(process.argv[1]);
    const archive = process.argv[2], out = process.argv[3];
    let ok = false;
    for (const url of urls) {
      console.error("crux: downloading " + url);
      try {
        execSync(\`curl -fSL --retry 3 -o "\${archive}" "\${url}"\`, { stdio: "inherit" });
        ok = fs.existsSync(archive) && fs.statSync(archive).size > 10000;
        if (ok) break;
      } catch (_) {
        try {
          execSync(\`wget -qO "\${archive}" "\${url}"\`, { stdio: "inherit" });
          ok = fs.existsSync(archive) && fs.statSync(archive).size > 10000;
          if (ok) break;
        } catch (_) {}
      }
    }
    if (!ok) process.exit(1);
    if (archive.endsWith(".zip")) {
      if (process.platform === "win32") execSync(\`tar -xf "\${archive}" -C "\${out}"\`);
      else execSync(\`unzip -o "\${archive}" -d "\${out}"\`);
    } else {
      execSync(\`tar -xzf "\${archive}" -C "\${out}"\`);
    }
  `;
  const tmpArchive = path.join(dest, asset);
  const r = spawnSync(process.execPath, ["-e", fetchScript, JSON.stringify(urls), tmpArchive, dest], { stdio: "inherit" });
  if (r.status !== 0 || !fs.existsSync(exe)) {
    console.error("crux: download failed. Install manually: cargo install crux-finder");
    console.error("or grab an asset from https://github.com/" + REPO + "/releases");
    process.exit(1);
  }
  if (process.platform !== "win32") {
    fs.chmodSync(exe, 0o755);
  }
  return exe;
}

const bin = findBinary() || download();
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error && r.error.code === "ENOENT") {
  console.error(`crux: failed to execute ${bin}`);
  process.exit(1);
}
process.exit(r.status ?? 1);
