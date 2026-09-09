// Shared development-only provenance. Run from the repository using the pinned Node.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';

export const workspace = path.dirname(fileURLToPath(import.meta.url));
export const root = path.resolve(workspace, '../../..');
export const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export function writeJson(file, value) {
  fs.mkdirSync(path.dirname(file), {recursive: true});
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n');
}
export function provenance(name) {
  const packageDir = path.join(workspace, 'node_modules', name);
  const metadata = JSON.parse(fs.readFileSync(path.join(packageDir, 'package.json')));
  const files = fs.readdirSync(path.join(packageDir, 'src'), {recursive: true})
    .filter(file => fs.statSync(path.join(packageDir, 'src', file)).isFile()).sort();
  return {
    name, version: metadata.version, license: metadata.license,
    source: `https://github.com/d3/${name}/tree/v${metadata.version}`,
    lock_sha256: digest(fs.readFileSync(path.join(workspace, 'package-lock.json'))),
    source_files: Object.fromEntries(files.map(file => ['src/' + file, digest(fs.readFileSync(path.join(packageDir, 'src', file)))])),
    node: process.versions.node, timezone: 'UTC', locale: 'en-US',
  };
}
export function retainLicense(name, directory) {
  fs.mkdirSync(directory, {recursive: true});
  fs.copyFileSync(path.join(workspace, 'node_modules', name, 'LICENSE'), path.join(directory, 'LICENSE'));
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const packages = JSON.parse(fs.readFileSync(path.join(workspace, 'package.json'))).dependencies;
  const rows = [];
  for (const [name, version] of Object.entries(packages)) {
    const module = await import(name);
    const identity = provenance(name);
    if (identity.version !== version) throw Error(`${name}: wrong reference version`);
    rows.push({...identity, exports: Object.keys(module).sort()});
  }
  writeJson(path.join(workspace, 'manifest.json'), {schema_version: 1, packages: rows});
  console.log(`PASS reference identities and export inventory: ${rows.length} pinned D3 modules.`);
}
