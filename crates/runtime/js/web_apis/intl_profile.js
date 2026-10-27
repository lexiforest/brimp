// Locale and timezone defaults for JavaScriptCore's host-provided Intl.
// JavaScriptCore has no per-context timezone API, so this shim injects the
// profile defaults into Intl constructors and Date locale/offset methods.
let __intlIdentity = { locale: 'en-US', timezone: 'UTC' };
let __intlInstalled = false;
let __intlOffsetFormatter = null;
let __intlOffsetFormatterZone = null;

function __intlOffsetMinutes(time) {
  if (__intlOffsetFormatterZone !== __intlIdentity.timezone) {
    __intlOffsetFormatterZone = __intlIdentity.timezone;
    __intlOffsetFormatter = new Intl.DateTimeFormat('en-US', {
      timeZone: __intlIdentity.timezone,
      hourCycle: 'h23',
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  }
  const parts = Object.fromEntries(
    __intlOffsetFormatter
      .formatToParts(new Date(time))
      .filter((part) => part.type !== 'literal')
      .map((part) => [part.type, Number(part.value)]),
  );
  const asUTC = Date.UTC(
    parts.year,
    parts.month - 1,
    parts.day,
    parts.hour,
    parts.minute,
    parts.second,
  );
  return Math.round((time - asUTC) / 60000);
}

function __installIntlProfile(identity) {
  __intlIdentity = { locale: identity.locale, timezone: identity.timezone };
  if (__intlInstalled) return;
  __intlInstalled = true;

  const wrap = (name, makeArguments) => {
    const Original = Intl[name];
    if (typeof Original !== 'function') return;
    function Wrapped(locales, options) {
      const [nextLocales, nextOptions] = makeArguments(locales, options);
      return Reflect.construct(Original, [nextLocales, nextOptions], new.target ?? Wrapped);
    }
    Object.setPrototypeOf(Wrapped, Original);
    Wrapped.prototype = Original.prototype;
    Object.defineProperty(Wrapped, 'name', { value: name, configurable: true });
    Object.defineProperty(Wrapped, 'length', { value: Original.length, configurable: true });
    Object.defineProperty(Wrapped.prototype, 'constructor', {
      value: Wrapped,
      writable: true,
      enumerable: false,
      configurable: true,
    });
    Object.defineProperty(Intl, name, {
      value: Wrapped,
      writable: true,
      enumerable: false,
      configurable: true,
    });
  };
  const optionsWithTimezone = (options) => {
    const next = options === undefined || options === null ? {} : { ...options };
    if (next.timeZone === undefined) next.timeZone = __intlIdentity.timezone;
    return next;
  };
  wrap('DateTimeFormat', (locales, options) => [
    locales === undefined ? __intlIdentity.locale : locales,
    optionsWithTimezone(options),
  ]);
  for (const name of [
    'NumberFormat',
    'Collator',
    'PluralRules',
    'RelativeTimeFormat',
    'ListFormat',
    'DisplayNames',
    'Segmenter',
  ]) {
    wrap(name, (locales, options) => [
      locales === undefined ? __intlIdentity.locale : locales,
      options === undefined ? {} : options,
    ]);
  }

  const proto = Date.prototype;
  const toLocaleString = proto.toLocaleString;
  proto.toLocaleString = function (locales, options) {
    return toLocaleString.call(
      this,
      locales === undefined ? __intlIdentity.locale : locales,
      optionsWithTimezone(options),
    );
  };
  const toLocaleDateString = proto.toLocaleDateString;
  proto.toLocaleDateString = function (locales, options) {
    return toLocaleDateString.call(
      this,
      locales === undefined ? __intlIdentity.locale : locales,
      optionsWithTimezone(options),
    );
  };
  const toLocaleTimeString = proto.toLocaleTimeString;
  proto.toLocaleTimeString = function (locales, options) {
    return toLocaleTimeString.call(
      this,
      locales === undefined ? __intlIdentity.locale : locales,
      optionsWithTimezone(options),
    );
  };
  proto.getTimezoneOffset = function () {
    const time = this.getTime();
    return Number.isFinite(time) ? __intlOffsetMinutes(time) : NaN;
  };
}

Object.defineProperty(globalThis, '__brimpInstallIntlProfile', {
  value: __installIntlProfile,
  writable: false,
  enumerable: false,
  configurable: true,
});
