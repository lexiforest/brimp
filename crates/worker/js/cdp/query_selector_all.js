Array.from(
    globalThis.__brimpCdpRemoteObjects
        ?.backendNodes
        ?.get(__NODE_ID__)
        ?.querySelectorAll(__SELECTOR__) ?? [],
)
