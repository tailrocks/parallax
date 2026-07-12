#!/usr/bin/env node

if (!process.versions.bun) {
  throw new Error("package scripts must execute with Bun")
}

console.log(`bun-runtime=${process.versions.bun}`)
