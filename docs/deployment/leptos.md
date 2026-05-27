# Leptos Deployment

## Use When

Use CSR for static hosting and keep the website crate separate from the main workspace dependency graph.

## Command Or Configuration

```toml
leptos = { version = "0.8", default-features = false, features = ["csr"] }
```

## Notes

- Keep assets small and avoid vendoring large models into published crates.
- Surface renderer initialization errors to the user.
- Use CI checks for every target you advertise.
