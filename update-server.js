const express = require('express');
const path = require('path');
const fs = require('fs');
const app = express();
const PORT = 3000;

// Serve static files from the bundle directory
const bundlePath = path.join(__dirname, 'guardian-ui', 'src-tauri', 'target', 'release', 'bundle');
app.use('/downloads', express.static(bundlePath));

// Update endpoint
app.get('/updates/:target/:arch/:current_version', (req, res) => {
  const { target, arch, current_version } = req.params;
  
  console.log(`Update check: ${target}/${arch} from version ${current_version}`);
  
  // For testing, always return a newer version
  const newVersion = "1.0.1";
  const currentVersion = current_version;
  
  // Compare versions (simple string comparison for testing)
  if (newVersion > currentVersion) {
    const updateResponse = {
      version: newVersion,
      notes: "Test update with bug fixes and improvements",
      pub_date: new Date().toISOString(),
      platforms: {
        [`${target}-${arch}`]: {
          signature: "dGVzdF9zaWduYXR1cmVfZm9yX2xvY2FsX3Rlc3Rpbmc=", // Base64 encoded test signature
          url: `http://localhost:${PORT}/downloads/nsis/Guardian_${newVersion}_x64-setup.exe`
        }
      }
    };
    
    console.log('Update available:', updateResponse);
    res.json(updateResponse);
  } else {
    console.log('No update available');
    res.status(204).send(); // No content - no update available
  }
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// List available versions
app.get('/versions', (req, res) => {
  const versions = [
    { version: "1.0.0", date: "2024-01-15T10:00:00Z", notes: "Initial release" },
    { version: "1.0.1", date: "2024-01-15T11:00:00Z", notes: "Test update with improvements" }
  ];
  res.json(versions);
});

app.listen(PORT, () => {
  console.log(`🚀 Update server running on http://localhost:${PORT}`);
  console.log(`📁 Serving files from: ${bundlePath}`);
  console.log(`🔍 Health check: http://localhost:${PORT}/health`);
  console.log(`📋 Available versions: http://localhost:${PORT}/versions`);
  console.log(`\n💡 To test updates:`);
  console.log(`   1. Start this server: node update-server.js`);
  console.log(`   2. Install your app`);
  console.log(`   3. Change version in tauri.conf.json to 1.0.0`);
  console.log(`   4. Rebuild and install`);
  console.log(`   5. App will detect update to 1.0.1`);
});
