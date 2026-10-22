function installPersona(persona) {
    const defaults = {
        userAgent: persona.base_headers.user_agent,
        appVersion: persona.base_headers.user_agent.replace(/^Mozilla\//, ''),
        platform: persona.platform,
        language: persona.locale,
        languages: Object.freeze(persona.languages.slice()),
        cookieEnabled: true,
        onLine: true,
    };
    const values = {...defaults};
    for (const name of Object.keys(defaults)) {
        const descriptor = Object.getOwnPropertyDescriptor({get [name]() { return values[name]; }}, name);
        Object.defineProperty(Navigator.prototype, name, descriptor);
    }
    return serialized => {
        const overrides = JSON.parse(serialized);
        for (const [name, value] of Object.entries(overrides)) {
            if (!(name in defaults)) continue;
            const previous = values[name];
            values[name] = value === null ? defaults[name] : name === 'languages' ? Object.freeze(Array.from(value, String)) : name === 'onLine' ? Boolean(value) : String(value);
            if (name === 'onLine' && previous !== values[name]) globalThis.dispatchEvent(new Event(values[name] ? 'online' : 'offline'));
        }
        return true;
    };
}
