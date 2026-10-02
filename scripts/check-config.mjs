import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const readJson = async (path) => JSON.parse(await readFile(new URL(path, import.meta.url), 'utf8'));
const projectRoot = new URL('../', import.meta.url);
const config = JSON.parse(await readFile(new URL('src-tauri/tauri.conf.json', projectRoot), 'utf8'));
const capability = JSON.parse(await readFile(new URL('src-tauri/capabilities/whatsapp-remote.json', projectRoot), 'utf8'));
const cargo = await readFile(new URL('src-tauri/Cargo.toml', projectRoot), 'utf8');
const bridge = await readFile(new URL('src-tauri/src/notification_bridge.js', projectRoot), 'utf8');

assert.equal(config.identifier, 'com.orbitratech.whatsapplite');
assert.deepEqual(config.app.security.capabilities, ['whatsapp-remote']);
assert.deepEqual(capability.remote.urls, ['https://web.whatsapp.com']);
assert(capability.permissions.includes('notification:allow-notify'));
assert(capability.permissions.includes('core:window:allow-show'));
assert(cargo.includes('tauri-plugin-autostart'));
assert(cargo.includes('tauri-plugin-global-shortcut'));
assert(cargo.includes('tauri-plugin-notification'));
assert(bridge.includes("window.location.origin !== 'https://web.whatsapp.com'"));
assert(config.bundle.targets.includes('msi'));
assert(config.bundle.targets.includes('dmg'));
assert(config.bundle.targets.includes('deb'));
assert(config.bundle.targets.includes('rpm'));

console.log('Static project configuration checks passed.');
