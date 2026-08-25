"use strict";
// Best-effort: pre-download the binary so first run is instant.
// If this fails (no network, blocked scripts), bin/crux.js downloads on demand.
const { spawnSync } = require("child_process");
const path = require("path");

const r = spawnSync(process.execPath, [path.join(__dirname, "bin", "crux.js"), "--help"], {
  stdio: "ignore",
});
process.exit(r.error ? 0 : 0);
