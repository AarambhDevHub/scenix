# Load Assets From URL

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Enable the loader `http` feature and use URL loading for remote asset bytes when supported by the target.

## Example

```toml
scenix-loader = { version = "1", features = ["http"] }
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
