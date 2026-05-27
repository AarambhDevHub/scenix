# Tauri Deployment

## Use When

Use Scenix CPU crates normally and integrate renderer/WASM decisions with the Tauri window strategy.

## Command Or Configuration

```toml
scenix = { version = "1", features = ["renderer"] }
```

## Notes

- Keep assets small and avoid vendoring large models into published crates.
- Surface renderer initialization errors to the user.
- Use CI checks for every target you advertise.
