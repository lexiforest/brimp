// Installer source: pass Plugin.fromFile / Plugin.from_file this file.
function installRealmLibrary({ global, options }) {
  global.textTools = Object.freeze({
    normalize(value) {
      return String(value).normalize(options.form ?? 'NFC');
    },
  });
  return () => {
    delete global.textTools;
  };
}
