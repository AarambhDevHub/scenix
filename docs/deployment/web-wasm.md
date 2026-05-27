# Web WASM Deployment

## Use When

Compile browser code for `wasm32-unknown-unknown` and serve generated static files.

## Command Or Configuration

```sh
rustup target add wasm32-unknown-unknown
```

## Notes

- Keep assets small and avoid vendoring large models into published crates.
- Surface renderer initialization errors to the user.
- Use CI checks for every target you advertise.
