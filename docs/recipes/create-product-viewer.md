# Create Product Viewer

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Combine glTF loading, orbit camera, raycaster selection, helpers, and optional post-processing.

## Example

```toml
scenix = { version = "1", features = ["loader", "renderer", "post"] }
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
