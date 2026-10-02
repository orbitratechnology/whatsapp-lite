import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

const sent = [];
let activateNotification;
let focused = 0;
const nativeNotifications = {
  onAction(callback) {
    activateNotification = callback;
    return Promise.resolve(() => {});
  },
  async isPermissionGranted() { return true; },
  async requestPermission() { return 'granted'; },
  async sendNotification(payload) { sent.push(payload); },
};
class NativeNotification extends EventTarget {
  static maxActions = 2;
  static get permission() { return 'granted'; }
}
const window = {
  location: { origin: 'https://web.whatsapp.com' },
  Notification: NativeNotification,
  __TAURI__: {
    notification: nativeNotifications,
    window: {
      getCurrentWindow: () => ({
        unminimize: async () => { focused += 1; },
        show: async () => { focused += 1; },
        setFocus: async () => { focused += 1; },
      }),
    },
  },
};
const source = await readFile(new URL('../src-tauri/src/notification_bridge.js', import.meta.url), 'utf8');
vm.runInNewContext(source, { window, Event, EventTarget, console, queueMicrotask, TypeError, Object, String, Boolean, Date });

const message = new window.Notification('New chat', { body: 'Hello from a test' });
assert(message instanceof NativeNotification, 'the wrapper retains Notification prototype compatibility');
await new Promise((resolve) => setTimeout(resolve, 0));
assert.equal(JSON.stringify(sent), JSON.stringify([{ title: 'New chat', body: 'Hello from a test' }]));
assert.equal(typeof activateNotification, 'function', 'notification activation handler is registered');
activateNotification();
assert.equal(focused, 3, 'notification activation restores and focuses the app');

window.__WHATSAPP_LITE_NOTIFICATIONS_ENABLED__ = false;
new window.Notification('Muted', { body: 'Must not be sent' });
await new Promise((resolve) => setTimeout(resolve, 0));
assert.equal(sent.length, 1, 'disabled notifications are not forwarded');

console.log('Notification bridge mock tests passed.');
