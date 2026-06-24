"use strict";
const crypto = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const https = require("node:https");

const ALLOWED_HOSTS = new Set(["github.com", "objects.githubusercontent.com", "release-assets.githubusercontent.com"]);

function target() {
  const { platform, arch } = process;
  const cpu = arch === "arm64" ? "aarch64" : "x86_64";
  if (platform === "linux") return `${cpu}-unknown-linux-gnu`;
  if (platform === "darwin") return `${cpu}-apple-darwin`;
  if (platform === "win32") return `${cpu}-pc-windows-msvc`;
  return null;
}

function binName() {
  return process.platform === "win32" ? "cmt.exe" : "cmt";
}

function validateUrl(u) {
  let parsed;
  try {
    parsed = new URL(u);
  } catch {
    return false;
  }
  return parsed.protocol === "https:" && ALLOWED_HOSTS.has(parsed.hostname);
}

function download(u, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (!validateUrl(u)) {
      return reject(new Error(`blocked redirect to untrusted host: ${u}`));
    }
    https
      .get(u, (res) => {
        if (
          [301, 302, 307, 308].includes(res.statusCode) &&
          res.headers.location
        ) {
          if (redirects >= 5) {
            res.resume();
            return reject(new Error("too many redirects"));
          }
          res.resume();
          return resolve(download(res.headers.location, redirects + 1));
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`HTTP ${res.statusCode}`));
        }
        const chunks = [];
        const hash = crypto.createHash("sha256");
        res.on("data", (chunk) => {
          chunks.push(chunk);
          hash.update(chunk);
        });
        res.on("end", () =>
          resolve({ data: Buffer.concat(chunks), digest: hash.digest("hex") })
        );
        res.on("error", reject);
      })
      .on("error", reject);
  });
}

async function main() {
  const t = target();
  if (!t) return; // unsupported platform — skip silently

  const ext = process.platform === "win32" ? ".exe" : "";
  const assetName = `cmt-${t}${ext}`;
  const ver = require("./../package.json").version;
  const url = `https://github.com/mihai-ro/cmt/releases/download/@mihairo/cmt-v${ver}/${assetName}`;
  const dest = path.join(__dirname, "bin", binName());

  let checksums;
  try {
    checksums = JSON.parse(
      fs.readFileSync(path.join(__dirname, "checksums.json"), "utf8")
    );
  } catch {
    console.error("cmt: checksums.json missing — cannot verify binary integrity");
    process.exit(1);
  }
  const expected = checksums[assetName];
  if (!expected) {
    console.error(`cmt: no checksum entry for ${assetName}`);
    process.exit(1);
  }

  console.error(`cmt: downloading ${assetName}...`);
  let result;
  try {
    result = await download(url);
  } catch (err) {
    console.error(`cmt: download failed — ${err.message}`);
    process.exit(1);
  }

  if (result.digest !== expected) {
    console.error(`cmt: checksum mismatch for ${assetName}`);
    console.error(`  expected: ${expected}`);
    console.error(`  got:      ${result.digest}`);
    process.exit(1);
  }

  const tmp = path.join(os.tmpdir(), `cmt-install-${process.pid}${ext}`);
  try {
    fs.mkdirSync(path.join(__dirname, "bin"), { recursive: true });
    fs.writeFileSync(tmp, result.data, { mode: 0o755 });
    fs.renameSync(tmp, dest);
  } catch (err) {
    console.error(`cmt: install failed — ${err.message}`);
    try {
      fs.unlinkSync(tmp);
    } catch {}
    process.exit(1);
  }

  console.error(`cmt: installed ${dest}`);
}

main();
