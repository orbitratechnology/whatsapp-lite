(() => {
  if (window.location.origin !== 'https://web.whatsapp.com') return;
  if (window.__WHATSAPP_LITE_BRIDGE_INSTALLED__) return;

  const tauri = window.__TAURI__;
  const nativeNotifications = tauri && tauri.notification;
  const NativeNotification = window.Notification;
  if (!nativeNotifications || !NativeNotification) return;

  window.__WHATSAPP_LITE_BRIDGE_INSTALLED__ = true;
  window.__WHATSAPP_LITE_NOTIFICATIONS_ENABLED__ = true;
  let permission = 'default';

  const focusApp = () => {
    try {
      const mainWindow = tauri.window.getCurrentWindow();
      void mainWindow.unminimize();
      void mainWindow.show();
      void mainWindow.setFocus();
    } catch (_) {
      // The notification itself remains usable if a platform lacks click actions.
    }
  };

  if (typeof nativeNotifications.onAction === 'function') {
    nativeNotifications.onAction(() => focusApp()).catch(() => {});
  }

  const eventMethods = ['addEventListener', 'removeEventListener', 'dispatchEvent'];
  function WhatsAppLiteNotification(title, options = {}) {
    if (!new.target) throw new TypeError("Failed to construct 'Notification': Please use the 'new' operator.");
    this.title = String(title ?? 'WhatsApp');
    this.body = String(options.body ?? '');
    this.tag = String(options.tag ?? '');
    this.data = options.data;
    this.dir = options.dir ?? 'auto';
    this.lang = options.lang ?? '';
    this.requireInteraction = Boolean(options.requireInteraction);
    this.silent = Boolean(options.silent);
    this.renotify = Boolean(options.renotify);
    this.timestamp = options.timestamp ?? Date.now();
    this.actions = options.actions ?? [];
    this.onclick = null;
    this.onerror = null;
    this.onshow = null;
    this.onclose = null;
    this._eventTarget = new EventTarget();
    for (const method of eventMethods) {
      this[method] = this._eventTarget[method].bind(this._eventTarget);
    }

    const notification = this;
    queueMicrotask(async () => {
      if (!window.__WHATSAPP_LITE_NOTIFICATIONS_ENABLED__) return;
      try {
        let granted = await nativeNotifications.isPermissionGranted();
        if (!granted && permission !== 'denied') {
          permission = await nativeNotifications.requestPermission();
          granted = permission === 'granted';
        }
        if (!granted) {
          permission = 'denied';
          return;
        }
        permission = 'granted';
        await nativeNotifications.sendNotification({
          title: notification.title.slice(0, 180),
          body: notification.body.slice(0, 600),
        });
        const event = new Event('show');
        notification.dispatchEvent(event);
        if (typeof notification.onshow === 'function') notification.onshow.call(notification, event);
      } catch (error) {
        const event = new Event('error');
        notification.dispatchEvent(event);
        if (typeof notification.onerror === 'function') notification.onerror.call(notification, event);
        console.warn('WhatsApp Lite notification bridge failed:', error);
      }
    });
  }

  WhatsAppLiteNotification.prototype = Object.create(NativeNotification.prototype);
  Object.defineProperty(WhatsAppLiteNotification.prototype, 'constructor', {
    configurable: true,
    writable: true,
    value: WhatsAppLiteNotification,
  });
  Object.setPrototypeOf(WhatsAppLiteNotification, NativeNotification);
  Object.defineProperty(WhatsAppLiteNotification, 'permission', {
    configurable: true,
    get: () => permission,
  });
  WhatsAppLiteNotification.requestPermission = (callback) => {
    const request = (async () => {
      const granted = await nativeNotifications.isPermissionGranted();
      permission = granted ? 'granted' : await nativeNotifications.requestPermission();
      return permission;
    })();
    if (typeof callback === 'function') request.then(callback);
    return request;
  };
  WhatsAppLiteNotification.maxActions = NativeNotification.maxActions ?? 0;
  WhatsAppLiteNotification.prototype.close = function close() {
    const event = new Event('close');
    this.dispatchEvent(event);
    if (typeof this.onclose === 'function') this.onclose.call(this, event);
  };

  window.Notification = WhatsAppLiteNotification;
})();
