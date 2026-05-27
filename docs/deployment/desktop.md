# Desktop Deployment

## Use When

Build native apps with CPU authoring by default and optional `renderer` for `wgpu` output.

## Command Or Configuration

```sh
cargo build --release --features renderer
```

## Notes

- Keep assets small and avoid vendoring large models into published crates.
- Surface renderer initialization errors to the user.
- Use CI checks for every target you advertise.
