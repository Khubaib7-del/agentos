// Downloads the platform binary for this package's version from GitHub
// Releases and verifies its SHA-256 against checksums pinned at publish
// time. A mismatch aborts the install — no unverified code ever runs.
const fs = require("fs");
const path = require("path");
const crypto = require("crypto");

const pkg = require("./package.json");
const checksums = require("./checksums.json");

const TARGETS = {
  "win32-x64": "thruline-x86_64-pc-windows-msvc.exe",
  "linux-x64": "thruline-x86_64-unknown-linux-gnu",
  "darwin-x64": "thruline-x86_64-apple-darwin",
  "darwin-arm64": "thruline-aarch64-apple-darwin",
};

function targetAsset() {
  const key = `${process.platform}-${process.arch}`;
  const asset = TARGETS[key];
  if (!asset) {
    console.error(`thruline: no prebuilt binary for ${key}.`);
    console.error(
      "Build from source instead: cargo install --git https://github.com/Khubaib7-del/thruline thruline-cli"
    );
    process.exit(1);
  }
  return asset;
}

function binPath() {
  const ext = process.platform === "win32" ? ".exe" : "";
  return path.join(__dirname, "bin", `thruline-bin${ext}`);
}

async function install() {
  const asset = targetAsset();
  const expected = checksums[asset];
  if (!expected) {
    console.error(`thruline: no pinned checksum for ${asset} — refusing to install.`);
    process.exit(1);
  }
  const url = `https://github.com/Khubaib7-del/thruline/releases/download/v${pkg.version}/${asset}`;
  console.log(`thruline: downloading ${url}`);
  const res = await fetch(url);
  if (!res.ok) {
    console.error(`thruline: download failed: ${res.status} ${res.statusText}`);
    process.exit(1);
  }
  const buf = Buffer.from(await res.arrayBuffer());
  const actual = crypto.createHash("sha256").update(buf).digest("hex");
  if (actual !== expected) {
    console.error("thruline: SHA-256 mismatch — refusing to install.");
    console.error(`  expected ${expected}`);
    console.error(`  got      ${actual}`);
    process.exit(1);
  }
  const dest = binPath();
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.writeFileSync(dest, buf);
  if (process.platform !== "win32") fs.chmodSync(dest, 0o755);
  console.log(`thruline: installed ${asset} (sha256 verified)`);
}

module.exports = { install, binPath };

if (require.main === module) {
  install().catch((e) => {
    console.error(`thruline: ${e.message}`);
    process.exit(1);
  });
}
