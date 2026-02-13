#!/usr/bin/env bun

import { chmodSync } from 'node:fs';

const packageJSON = await Bun.file(new URL('./package.json', import.meta.url)).json();

const matrix = {
  platform: {
    'darwin': {
      name: 'macos',
      arch: {
        'arm64': { name: 'arm64' },
        'x64': { name: 'x64' },
      }
    },
    'linux': {
      name: 'linux',
      arch: {
        'arm64': { name: 'arm64' },
        'arm': { name: 'arm' },
        'x64': { name: 'x64' },
        'x86': { name: 'x86' },
        'ppc64': { name: 'powerpc64' },
      }
    },
    'win32': {
      name: 'windows',
      arch: {
        'arm64': { name: 'arm64' },
        'x64': { name: 'x64' },
        'x86': { name: 'x86' },
        'ia32': { name: 'x86' },
      }
    },
  },
}

// Determine the URL of the file.
const platform = matrix.platform[process.platform];
const arch = platform?.arch[process.arch];

if (!platform || !platform.name || !arch || !arch.name) {
  console.error(
    `Cannot install tree-sitter-cli for platform ${process.platform}, architecture ${process.arch}`
  );
  process.exit(1);
}

const releaseURL = `https://github.com/tree-sitter/tree-sitter/releases/download/v${packageJSON.version}`;
const assetName = `tree-sitter-${platform.name}-${arch.name}.gz`;
const assetURL = `${releaseURL}/${assetName}`;

// Remove previously-downloaded files.
const executableName = process.platform === 'win32' ? 'tree-sitter.exe' : 'tree-sitter';
const executableFile = Bun.file(executableName);
if (await executableFile.exists()) {
  await executableFile.delete();
}

// Download the compressed file.
console.log(`Downloading ${assetURL}`);
const response = await fetch(assetURL);
if (!response.ok) {
  console.error([
    'Download failed',
    '',
    `url: ${assetURL}`,
    `status: ${response.status}`,
    '',
  ].join('\n'));
  process.exit(1);
}

const decompressed = Bun.gunzipSync(await response.bytes());
await Bun.write(executableName, decompressed);
chmodSync(executableName, '755');
