"use strict";

const { platform, arch } = process;
const isWindows = platform === "win32";
const isMusl = () => {
  if (typeof process.report === "object" && typeof process.report.getReport === "function") {
    const report = process.report.getReport();
    if (report && report.header && typeof report.header.glibcVersion === "string") {
      return false;
    }
    if (report && report.header && report.header.glibcVersion === undefined) {
      return true;
    }
  }
  try {
    require("fs").statSync("/lib64/ld-musl-x86_64.so.1");
    return true;
  } catch {
    return false;
  }
};

let nativeBinding = null;
const loadErrors = [];

function requireOptionalDependency(name) {
  try {
    return require(name);
  } catch (e) {
    loadErrors.push(`Optional dependency ${name}: ${e.message}`);
    return null;
  }
}

const tryLoadBinding = () => {
  const targets = [
    [
      "linux",
      "x64",
      "gnu",
      "./ts-pack-core-node.linux-x64-gnu.node",
      "ts-pack-core-node-linux-x64-gnu",
    ],
    [
      "linux",
      "x64",
      "musl",
      "./ts-pack-core-node.linux-x64-musl.node",
      "ts-pack-core-node-linux-x64-musl",
    ],
    [
      "linux",
      "arm64",
      "gnu",
      "./ts-pack-core-node.linux-arm64-gnu.node",
      "ts-pack-core-node-linux-arm64-gnu",
    ],
    [
      "linux",
      "arm64",
      "musl",
      "./ts-pack-core-node.linux-arm64-musl.node",
      "ts-pack-core-node-linux-arm64-musl",
    ],
    ["darwin", "x64", null, "./ts-pack-core-node.darwin-x64.node", "ts-pack-core-node-darwin-x64"],
    [
      "darwin",
      "arm64",
      null,
      "./ts-pack-core-node.darwin-arm64.node",
      "ts-pack-core-node-darwin-arm64",
    ],
    [
      "win32",
      "x64",
      null,
      "./ts-pack-core-node.win32-x64-msvc.node",
      "ts-pack-core-node-win32-x64-msvc",
    ],
    [
      "win32",
      "arm64",
      null,
      "./ts-pack-core-node.win32-arm64-msvc.node",
      "ts-pack-core-node-win32-arm64-msvc",
    ],
  ];

  for (const [plat, a, abi, localPath, optionalDep] of targets) {
    if (platform !== plat || arch !== a) {
      continue;
    }

    if (plat === "linux" && abi) {
      const isCurMusl = isMusl();
      if ((abi === "musl") !== isCurMusl) {
        continue;
      }
    }

    try {
      nativeBinding = require(localPath);
      if (nativeBinding) {
        return;
      }
    } catch (e) {
      loadErrors.push(e.message);
    }

    try {
      const optBinding = requireOptionalDependency(optionalDep);
      if (optBinding) {
        nativeBinding = optBinding;
        return;
      }
    } catch (e) {
      loadErrors.push(e.message);
    }
  }
};

tryLoadBinding();

if (!nativeBinding) {
  throw new Error(
    `Failed to load native binding for ${platform}-${arch}. Errors: ${loadErrors.join(", ")}`,
  );
}

module.exports = nativeBinding;
