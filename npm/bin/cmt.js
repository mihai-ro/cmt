#!/usr/bin/env node
"use strict";
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const fs = require("node:fs");

function binName() {
  return process.platform === "win32" ? "cmt.exe" : "cmt";
}

const bin = path.join(__dirname, binName());
if (!fs.existsSync(bin)) {
  console.error("cmt: no prebuilt binary found for this platform. Run: node npm/postinstall.js");
  process.exit(1);
}
const res = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
process.exit(res.status === null ? 1 : res.status);
