# GitHub Pages Deployment

## Use When

Build the static Leptos CSR website with `/scenix/` as the public URL.

## Command Or Configuration

```sh
cd website
trunk build --release --public-url /scenix/
```

## Notes

- Keep assets small and avoid vendoring large models into published crates.
- Surface renderer initialization errors to the user.
- Use CI checks for every target you advertise.
