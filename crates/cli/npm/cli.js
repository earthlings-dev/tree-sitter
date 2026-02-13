#!/usr/bin/env bun

import { join } from 'node:path';

const executable = process.platform === 'win32'
  ? 'tree-sitter.exe'
  : 'tree-sitter';
const proc = Bun.spawn(
  [join(import.meta.dir, executable), ...process.argv.slice(2)],
  { stdio: ['inherit', 'inherit', 'inherit'] },
);
process.exitCode = await proc.exited;
