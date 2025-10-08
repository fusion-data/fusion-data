#!/usr/bin/env node

/**
 * Post-build script for hetuflow-sdk WASM package
 * Automatically adds dayjs dependency to the generated package.json
 */

const fs = require('fs');
const path = require('path');

// Path to the generated package.json
const pkgPath = path.join(__dirname, 'examples/wasm/pkg/package.json');

try {
  // Read the generated package.json
  const packageJson = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  
  // Add dayjs dependency if not already present
  if (!packageJson.dependencies) {
    packageJson.dependencies = {};
  }
  
  if (!packageJson.dependencies.dayjs) {
    packageJson.dependencies.dayjs = '^1.11.0';
    console.log('Added dayjs dependency to package.json');
  } else {
    console.log('dayjs dependency already exists in package.json');
  }
  
  // Write back the modified package.json
  fs.writeFileSync(pkgPath, JSON.stringify(packageJson, null, 2) + '\n');
  console.log('Successfully updated package.json');
  
} catch (error) {
  console.error('Error updating package.json:', error.message);
  process.exit(1);
}