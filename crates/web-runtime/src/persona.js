// Installs deterministic API-shape emulation from a fully resolved persona.
// Rendering, media processing, device access, and network services stay native
// or absent; this module only owns values that can be represented in JavaScript.
function installPersona(persona) {
    "use strict";

    const navigatorValues = persona.js;
    const network = persona.network;
    const feature = name => persona.features[name] !== false;
    const defineValue = (target, name, value, enumerable = true) => {
        Object.defineProperty(target, name, {
            value,
            enumerable,
            configurable: true,
        });
    };

    const navigatorProperties = {
        userAgent: { value: network.user_agent, enumerable: true },
        appVersion: { value: navigatorValues.app_version, enumerable: true },
        platform: { value: navigatorValues.platform, enumerable: true },
        language: { value: navigatorValues.language, enumerable: true },
        languages: { value: Object.freeze(navigatorValues.languages.slice()), enumerable: true },
        hardwareConcurrency: { value: navigatorValues.hardware_concurrency, enumerable: true },
        maxTouchPoints: { value: navigatorValues.max_touch_points, enumerable: true },
        doNotTrack: { value: navigatorValues.do_not_track, enumerable: true },
        vendor: { value: navigatorValues.vendor, enumerable: true },
        productSub: { value: navigatorValues.product_sub, enumerable: true },
        pdfViewerEnabled: { value: navigatorValues.pdf_viewer_enabled, enumerable: true },
        webdriver: { value: persona.automation.webdriver, enumerable: true },
    };
    if (navigatorValues.oscpu) {
        navigatorProperties.oscpu = { value: navigatorValues.oscpu, enumerable: true };
    }
    if (feature("device_memory")) {
        navigatorProperties.deviceMemory = {
            value: navigatorValues.device_memory_gb,
            enumerable: true,
        };
    }
    if (navigatorValues.expose_global_privacy_control) {
        navigatorProperties.globalPrivacyControl = {
            value: navigatorValues.global_privacy_control,
            enumerable: true,
        };
    }
    if (feature("network_information")) {
        navigatorProperties.connection = {
            value: Object.freeze({
                type: navigatorValues.connection_type,
                rtt: navigatorValues.connection_rtt_ms,
                downlink: Number(navigatorValues.connection_downlink_mbps),
                effectiveType: navigatorValues.connection_effective_type,
                saveData: navigatorValues.connection_save_data,
            }),
            enumerable: true,
        };
    }
    if (feature("user_agent_data")) {
        const majorVersion = persona.browser_version.split(".")[0];
        const brands = Object.freeze([
            Object.freeze({ brand: persona.browser_name, version: majorVersion }),
        ]);
        const fullVersionList = Object.freeze([
            Object.freeze({
                brand: persona.browser_name,
                version: persona.browser_version,
            }),
        ]);
        navigatorProperties.userAgentData = {
            value: Object.freeze({
                brands,
                mobile: network.sec_ch_ua_mobile === "?1",
                platform: persona.platform_name,
                getHighEntropyValues(hints) {
                    const values = {
                        architecture: navigatorValues.ua_architecture,
                        bitness: navigatorValues.ua_bitness,
                        model: navigatorValues.ua_model,
                        platformVersion: navigatorValues.ua_platform_version,
                        uaFullVersion: persona.browser_version,
                        fullVersionList,
                    };
                    const selected = {
                        brands,
                        mobile: network.sec_ch_ua_mobile === "?1",
                        platform: persona.platform_name,
                    };
                    for (const hint of hints) {
                        if (Object.hasOwn(values, hint)) selected[hint] = values[hint];
                    }
                    return Promise.resolve(selected);
                },
                toJSON() {
                    return {
                        brands,
                        mobile: network.sec_ch_ua_mobile === "?1",
                        platform: persona.platform_name,
                    };
                },
            }),
            enumerable: true,
        };
    }
    Object.defineProperties(navigator, navigatorProperties);

    const screenValues = persona.screen;
    const screenObject = Object.freeze({
        width: screenValues.width,
        height: screenValues.height,
        availWidth: screenValues.avail_width,
        availHeight: screenValues.avail_height,
        availLeft: screenValues.avail_left,
        availTop: screenValues.avail_top,
        left: screenValues.left,
        top: screenValues.top,
        colorDepth: screenValues.color_depth,
        pixelDepth: screenValues.pixel_depth,
        isExtended: screenValues.is_extended,
        orientation: Object.freeze({
            type: screenValues.orientation_type,
            angle: screenValues.orientation_angle,
        }),
    });
    defineValue(globalThis, "screen", screenObject, false);

    const windowValues = persona.window;
    Object.defineProperties(globalThis, {
        outerWidth: { value: windowValues.outer_width, configurable: true },
        outerHeight: { value: windowValues.outer_height, configurable: true },
        screenX: { value: windowValues.screen_x, configurable: true },
        screenY: { value: windowValues.screen_y, configurable: true },
        screenLeft: { value: windowValues.screen_x, configurable: true },
        screenTop: { value: windowValues.screen_y, configurable: true },
    });

    const makeNamedArray = (entries, names) => {
        const array = entries.slice();
        defineValue(array, "item", index => array[index] ?? null, false);
        defineValue(array, "namedItem", name => names[name] ?? null, false);
        for (const [name, value] of Object.entries(names)) {
            if (!(name in array)) defineValue(array, name, value, false);
        }
        return array;
    };

    const mimeByType = {};
    const pluginByName = {};
    const pluginEntries = persona.plugins.entries.map(plugin => {
        const value = {
            name: plugin.name,
            filename: plugin.filename,
            description: plugin.description,
        };
        const mimeNames = {};
        const mimeEntries = plugin.mime_types.map(mime => {
            const mimeValue = {
                type: mime.type,
                suffixes: mime.suffixes,
                description: mime.description,
            };
            mimeNames[mime.type] = mimeValue;
            if (!(mime.type in mimeByType)) mimeByType[mime.type] = mimeValue;
            return mimeValue;
        });
        const mimeArray = Object.freeze(makeNamedArray(mimeEntries, mimeNames));
        for (let index = 0; index < mimeArray.length; index += 1) {
            defineValue(value, index, mimeArray[index]);
        }
        defineValue(value, "length", mimeArray.length);
        defineValue(value, "item", index => mimeArray[index] ?? null, false);
        defineValue(value, "namedItem", name => mimeNames[name] ?? null, false);
        pluginByName[plugin.name] = value;
        return Object.freeze(value);
    });
    const mimeEntries = Object.values(mimeByType).map(mime => Object.freeze(mime));
    const plugins = makeNamedArray(pluginEntries, pluginByName);
    defineValue(plugins, "refresh", () => undefined, false);
    defineValue(navigator, "plugins", Object.freeze(plugins));
    defineValue(navigator, "mimeTypes", Object.freeze(makeNamedArray(mimeEntries, mimeByType)));

    if (feature("notifications")) {
        function Notification(title, options = {}) {
            this.title = String(title);
            this.body = options.body === undefined ? "" : String(options.body);
            this.close = () => undefined;
        }
        Object.defineProperty(Notification, "permission", {
            get: () => navigatorValues.notification_permission,
        });
        Notification.requestPermission = callback => {
            const permission = navigatorValues.notification_permission;
            if (typeof callback === "function") callback(permission);
            return Promise.resolve(permission);
        };
        defineValue(globalThis, "Notification", Notification, false);
    } else {
        delete globalThis.Notification;
    }

    if (feature("permissions") && navigatorValues.permissions_enabled) {
        const permissions = Object.freeze({
            query(descriptor) {
                const state = descriptor && descriptor.name === "notifications"
                    ? navigatorValues.notification_permission
                    : "granted";
                return Promise.resolve(Object.freeze({
                    state,
                    onchange: null,
                    addEventListener() {},
                    removeEventListener() {},
                }));
            },
        });
        defineValue(navigator, "permissions", permissions);
    } else {
        delete navigator.permissions;
    }

    if (feature("bluetooth") && navigatorValues.bluetooth_enabled) {
        defineValue(navigator, "bluetooth", Object.freeze({
            getAvailability: () => Promise.resolve(navigatorValues.bluetooth_available),
        }));
    } else {
        delete navigator.bluetooth;
    }

    const mediaValues = persona.media;
    if (navigatorValues.media_devices_enabled) {
        const devices = [];
        const addDevices = (kind, count) => {
            for (let index = 0; index < count; index += 1) {
                devices.push(Object.freeze({
                    deviceId: `${kind}-${index + 1}`,
                    groupId: `${kind}-group-${index + 1}`,
                    kind,
                    label: `${kind} ${index + 1}`,
                    toJSON() {
                        return {
                            deviceId: this.deviceId,
                            groupId: this.groupId,
                            kind: this.kind,
                            label: this.label,
                        };
                    },
                }));
            }
        };
        addDevices("audioinput", mediaValues.audio_inputs);
        addDevices("videoinput", mediaValues.video_inputs);
        addDevices("audiooutput", mediaValues.audio_outputs);
        defineValue(navigator, "mediaDevices", Object.freeze({
            enumerateDevices: () => Promise.resolve(devices.slice()),
            getSupportedConstraints: () => Object.freeze({}),
        }));
    } else {
        delete navigator.mediaDevices;
    }

    function MediaSource() {
        throw new TypeError("Illegal constructor");
    }
    MediaSource.isTypeSupported = type => mediaValues.media_source_supported_types.includes(String(type));
    defineValue(globalThis, "MediaSource", MediaSource, false);

    const decodingCapabilities = new Map(
        mediaValues.decoding_capabilities.map(capability => [capability.content_type, capability]),
    );
    defineValue(navigator, "mediaCapabilities", Object.freeze({
        decodingInfo(configuration) {
            const contentType = configuration?.video?.contentType
                ?? configuration?.audio?.contentType
                ?? "";
            const capability = decodingCapabilities.get(contentType);
            return Promise.resolve(Object.freeze({
                supported: capability?.supported ?? false,
                smooth: capability?.smooth ?? false,
                powerEfficient: capability?.power_efficient ?? false,
                configuration,
            }));
        },
    }));

    const configuredVoices = persona.speech.voices.length > 0
        ? persona.speech.voices
        : mediaValues.speech_voices.map((name, index) => ({
            voice_uri: name,
            name,
            lang: navigatorValues.language,
            default: index === 0,
            local_service: true,
        }));
    const voices = configuredVoices.map(voice => Object.freeze({
        voiceURI: voice.voice_uri,
        name: voice.name,
        lang: voice.lang,
        default: voice.default,
        localService: voice.local_service,
    }));
    defineValue(globalThis, "speechSynthesis", Object.freeze({
        pending: false,
        speaking: false,
        paused: false,
        getVoices: () => voices.slice(),
        speak() {},
        cancel() {},
        pause() {},
        resume() {},
        addEventListener() {},
        removeEventListener() {},
    }), false);

    if (feature("geolocation")) {
        let nextWatchId = 1;
        const position = persona.geo.latitude !== null && persona.geo.longitude !== null
            ? Object.freeze({
                coords: Object.freeze({
                    latitude: Number(persona.geo.latitude),
                    longitude: Number(persona.geo.longitude),
                    accuracy: persona.geo.accuracy_meters ?? 0,
                    altitude: persona.geo.altitude === null ? null : Number(persona.geo.altitude),
                    altitudeAccuracy: persona.geo.altitude_accuracy_meters === null
                        ? null
                        : Number(persona.geo.altitude_accuracy_meters),
                    heading: persona.geo.heading_degrees === null
                        ? null
                        : Number(persona.geo.heading_degrees),
                    speed: persona.geo.speed_meters_per_second === null
                        ? null
                        : Number(persona.geo.speed_meters_per_second),
                }),
                timestamp: Date.now(),
            })
            : null;
        const deliverPosition = (success, error) => {
            if (position !== null) {
                if (typeof success === "function") success(position);
            } else if (typeof error === "function") {
                error(Object.freeze({ code: 2, message: "Position unavailable" }));
            }
        };
        defineValue(navigator, "geolocation", Object.freeze({
            getCurrentPosition: deliverPosition,
            watchPosition(success, error) {
                const id = nextWatchId;
                nextWatchId += 1;
                deliverPosition(success, error);
                return id;
            },
            clearWatch() {},
        }));
    } else {
        delete navigator.geolocation;
    }

    if (feature("battery")) {
        const batteryValues = persona.battery;
        const battery = Object.freeze({
            charging: batteryValues.charging,
            chargingTime: batteryValues.charging_time_seconds,
            dischargingTime: batteryValues.discharging_time_seconds ?? Infinity,
            level: batteryValues.level_percent / 100,
            onchargingchange: null,
            onchargingtimechange: null,
            ondischargingtimechange: null,
            onlevelchange: null,
            addEventListener() {},
            removeEventListener() {},
        });
        defineValue(navigator, "getBattery", () => Promise.resolve(battery));
    } else {
        delete navigator.getBattery;
    }

    const storageValues = persona.storage;
    const storageEstimate = () => {
        const estimate = { usage: storageValues.usage_bytes };
        if (storageValues.quota_bytes !== null) estimate.quota = storageValues.quota_bytes;
        return Object.freeze(estimate);
    };
    defineValue(navigator, "storage", Object.freeze({
        estimate: () => Promise.resolve(storageEstimate()),
    }));
    const legacyStorage = quota => Object.freeze({
        queryUsageAndQuota(successCallback) {
            if (typeof successCallback === "function") {
                successCallback(storageValues.usage_bytes, quota ?? 0);
            }
        },
        requestQuota(bytes, successCallback) {
            const granted = quota === null ? Number(bytes) : Math.min(Number(bytes), quota);
            if (typeof successCallback === "function") successCallback(granted);
        },
    });
    defineValue(navigator, "webkitTemporaryStorage", legacyStorage(storageValues.legacy_temporary_quota_bytes));
    defineValue(navigator, "webkitPersistentStorage", legacyStorage(storageValues.legacy_persistent_quota_bytes));

    if (feature("webrtc")) {
        const parseCodec = (codec, kind) => {
            const [identity, ...parameters] = codec.split(";");
            const parts = identity.split("/");
            const capability = {
                mimeType: `${parts[0]}/${parts[1]}`,
                clockRate: Number(parts[2] ?? (kind === "video" ? 90000 : 48000)),
            };
            if (kind === "audio" && parts[1].toLowerCase() === "opus") capability.channels = 2;
            if (parameters.length > 0) capability.sdpFmtpLine = parameters.join(";");
            return Object.freeze(capability);
        };
        const capabilities = kind => {
            const codecs = kind === "audio" ? persona.webrtc.audio_codecs : persona.webrtc.video_codecs;
            return Object.freeze({
                codecs: Object.freeze(codecs.map(codec => parseCodec(codec, kind))),
                headerExtensions: Object.freeze([]),
            });
        };
        function RTCRtpSender() {
            throw new TypeError("Illegal constructor");
        }
        RTCRtpSender.getCapabilities = capabilities;
        function RTCRtpReceiver() {
            throw new TypeError("Illegal constructor");
        }
        RTCRtpReceiver.getCapabilities = capabilities;
        defineValue(globalThis, "RTCRtpSender", RTCRtpSender, false);
        defineValue(globalThis, "RTCRtpReceiver", RTCRtpReceiver, false);
    } else {
        delete globalThis.RTCRtpSender;
        delete globalThis.RTCRtpReceiver;
    }

    const chromeValues = persona.chrome;
    if (chromeValues.exposed !== false) {
        const value = {};
        if (chromeValues.runtime !== false) value.runtime = Object.freeze({});
        if (chromeValues.app !== false) value.app = Object.freeze({});
        if (chromeValues.load_times !== false) value.loadTimes = () => Object.freeze({});
        if (chromeValues.csi !== false) value.csi = () => Object.freeze({});
        defineValue(globalThis, "chrome", Object.freeze(value), chromeValues.enumerable === true);
    } else {
        delete globalThis.chrome;
    }
}
