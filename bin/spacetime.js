#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

function getBinaryPath() {
  // Check for local development release/debug build
  const releasePath = path.join(__dirname, '..', 'target', 'release', 'spacetime');
  const debugPath = path.join(__dirname, '..', 'target', 'debug', 'spacetime');

  if (fs.existsSync(releasePath)) return releasePath;
  if (fs.existsSync(debugPath)) return debugPath;

  // Platform specific binary package fallback
  const platform = process.platform;
  const arch = process.arch;
  const binaryName = platform === 'win32' ? 'spacetime.exe' : 'spacetime';

  const bundledPath = path.join(__dirname, binaryName);
  if (fs.existsSync(bundledPath)) return bundledPath;

  console.error(`Error: Spacetime binary not found for ${platform}-${arch}. Please build with 'cargo build --release' or run 'curl -fsSL https://raw.githubusercontent.com/phasehumans/spacetime/main/install.sh | sh'.`);
  process.exit(1);
}

const binaryPath = getBinaryPath();
const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit'
});

child.on('exit', (code) => {
  process.exit(code || 0);
});
