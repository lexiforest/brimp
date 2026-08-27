# persona

`persona` owns Brimp's versioned browser-persona configuration, validation,
preset resolution, and deterministic seeds. It was migrated from Bimp's
`bimp-persona` crate; obsolete `browser` and `startup_url` aliases were removed.

```rust
use persona::PersonaConfig;

let config = PersonaConfig::from_json(r#"{
  "schema_version": 1,
  "transport": { "impersonation_profile": "chrome150" },
  "locale": "en-US",
  "viewport": { "width": 1280, "height": 720, "device_scale_factor": 2 }
}"#)?;
let resolved = config.resolve();
# Ok::<(), persona::PersonaConfigError>(())
```

The authoritative JSON schema, complete example, and field-by-field runtime
support matrix live in [`../../persona/`](../../persona/README.md). Parsing and
resolution support the complete schema. Runtime application is intentionally
smaller and is documented separately because Brimp does not patch Blitz or
JavaScriptCore for persona behavior.
