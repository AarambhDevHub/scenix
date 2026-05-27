# Materials Guide

Scenix material crates are GPU-free descriptions. The renderer converts them into stable preview uniforms and pipeline keys.

## Built-In Materials

- `PbrMaterial`: metallic-roughness material for standard examples.
- `PhysicalMaterial`: clearcoat, transmission, sheen, and advanced surface fields. v1 renders this through a stable preview path and documents advanced physical accuracy as a limitation.
- `UnlitMaterial`: constant color preview.
- `LambertMaterial`: diffuse lighting preview.
- `ToonMaterial`: banded preview shading.
- `WireframeMaterial`: debug preview material.
- `NormalMaterial`: normal visualization preview.

## Pipeline Keys

Pipeline keys keep renderer setup deterministic. The renderer caches pipelines and avoids shader/pipeline creation in the steady-state frame path.

## Texture Ownership

Textures stay CPU-side until registered with `Renderer::register_texture2d`. Loaders decode images into `Texture2D`; they do not upload to the GPU automatically.
