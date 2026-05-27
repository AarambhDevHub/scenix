# Dioxus Deployment

## Use When

Use Scenix as the scene/data layer and integrate rendering according to the Dioxus target and WebGPU availability.

## Command Or Configuration

```toml
scenix = { version = "1", features = ["wasm"] }
```

## Notes

- Keep assets small and avoid vendoring large models into published crates.
- Surface renderer initialization errors to the user.
- Use CI checks for every target you advertise.
