/**
 * Copy native bindings from Rust build to npm package locations
 * Supports both napi build output (.node files) and cargo build output (.dll/.so/.dylib)
 */

const fs = require('fs');
const path = require('path');

const projectRoot = path.join(__dirname, '..', '..');
const cargoReleaseDir = path.join(projectRoot, 'target', 'release');
const napiBuildDir = path.join(projectRoot, 'crates', 'agw-napi');
const npmPackagesDir = path.join(__dirname, '..', 'packages', '@agent-gateway');

const platforms = [
  { name: 'win32-x64', napiFile: 'agw-napi.node', cargoFile: 'agw_napi.dll', targetDir: 'node-win32-x64' },
];

function copyBindings() {
  let copied = 0;

  for (const platform of platforms) {
    const targetDir = path.join(npmPackagesDir, platform.targetDir, 'native');
    const target = path.join(targetDir, `agw-napi.${platform.name}.node`);

    if (!fs.existsSync(targetDir)) {
      fs.mkdirSync(targetDir, { recursive: true });
    }

    // Try napi build output first (preferred)
    const napiSource = path.join(napiBuildDir, platform.napiFile);
    if (fs.existsSync(napiSource)) {
      fs.copyFileSync(napiSource, target);
      console.log(`✓ Copied napi output: ${napiSource} -> ${target}`);
      copied++;
      continue;
    }

    // Fallback to cargo build output
    const cargoSource = path.join(cargoReleaseDir, platform.cargoFile);
    if (fs.existsSync(cargoSource)) {
      fs.copyFileSync(cargoSource, target);
      console.log(`✓ Copied cargo output: ${cargoSource} -> ${target}`);
      copied++;
      continue;
    }

    console.log(`⚠ Source not found for ${platform.name}`);
    console.log('  Run "napi build --release" or "cargo build --release -p agw-napi" first');
  }

  if (copied === 0) {
    console.error('❌ No native bindings were copied. Build the native addon first.');
    process.exit(1);
  }

  console.log(`\n✓ Copied ${copied}/${platforms.length} platform binding(s)`);
}

copyBindings();