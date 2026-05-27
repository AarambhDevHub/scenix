# Create 3D Portfolio Section

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Use the WASM wrapper or website pattern for a generated browser scene with clean fallback UI.

## Example

```sh
cd website
trunk build --release --public-url /scenix/
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
