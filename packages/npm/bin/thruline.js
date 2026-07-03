#!/usr/bin/env node
// Thin shim: ensures the verified binary exists (postinstall may have been
// skipped with --ignore-scripts), then passes everything through to it.
const fs = require("fs");
const { spawnSync } = require("child_process");
const { install, binPath } = require("../install.js");

async function main() {
  const bin = binPath();
  if (!fs.existsSync(bin)) {
    await install();
  }
  const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
  process.exit(result.status ?? 1);
}

main().catch((e) => {
  console.error(`thruline: ${e.message}`);
  process.exit(1);
});
