const fs = require('fs');
const path = require('path');

function copyIfExists(src, dest) {
  try {
    if (!fs.existsSync(src)) {
      console.warn(`[copy-binaries] Source not found: ${src}`);
      return false;
    }
    fs.copyFileSync(src, dest);
    console.log(`[copy-binaries] Copied ${src} -> ${dest}`);
    return true;
  } catch (e) {
    console.error(`[copy-binaries] Failed to copy ${src} -> ${dest}:`, e.message);
    return false;
  }
}

try {
  const here = __dirname; // .../guardian-ui/src-tauri/gen
  const uiDir = path.resolve(here, '..'); // .../guardian-ui/src-tauri
  const projectRoot = path.resolve(uiDir, '..'); // .../guardian-ui
  const repoRoot = path.resolve(projectRoot, '..'); // repo root

  // hostd.exe - copy with target triple name for Tauri
  const hostdSrc = path.join(repoRoot, 'hostd', 'target', 'release', 'hostd.exe');
  const hostdDest = path.join(uiDir, 'hostd-x86_64-pc-windows-msvc.exe');
  copyIfExists(hostdSrc, hostdDest);
  
  // Also copy as hostd.exe for compatibility
  const hostdDestCompat = path.join(uiDir, 'hostd.exe');
  copyIfExists(hostdSrc, hostdDestCompat);

  // gpu-worker.exe - copy with target triple name for Tauri
  const gpuSrc = path.join(repoRoot, 'gpu-worker', 'target', 'release', 'gpu-worker.exe');
  const gpuDest = path.join(uiDir, 'gpu-worker-x86_64-pc-windows-msvc.exe');
  copyIfExists(gpuSrc, gpuDest);
  
  // Also copy as gpu-worker.exe for compatibility
  const gpuDestCompat = path.join(uiDir, 'gpu-worker.exe');
  copyIfExists(gpuSrc, gpuDestCompat);

  // init_db.exe - copy with target triple name for Tauri
  const initDbSrc = path.join(repoRoot, 'hostd', 'target', 'release', 'init_db.exe');
  const initDbDest = path.join(uiDir, 'init_db-x86_64-pc-windows-msvc.exe');
  copyIfExists(initDbSrc, initDbDest);
  
  // Also copy as init_db.exe for compatibility
  const initDbDestCompat = path.join(uiDir, 'init_db.exe');
  copyIfExists(initDbSrc, initDbDestCompat);

  // configs
  const configsSrc = path.join(repoRoot, 'configs');
  const configsDest = path.join(uiDir, 'configs');
  if (fs.existsSync(configsSrc)) {
    fs.mkdirSync(configsDest, { recursive: true });
    for (const file of fs.readdirSync(configsSrc)) {
      const s = path.join(configsSrc, file);
      const d = path.join(configsDest, file);
      if (fs.statSync(s).isFile()) {
        copyIfExists(s, d);
      }
    }
  }
} catch (e) {
  console.error('[copy-binaries] Unexpected error:', e);
  process.exit(0); // don't fail build
}



