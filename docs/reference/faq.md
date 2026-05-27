# FAQ

## Is Scenix a game engine?

No. Scenix is a modular scene, rendering, loading, animation-bridge, and browser integration workspace. It does not provide ECS, physics, audio, or game-loop ownership.

## Do I need the renderer?

No. CPU authoring, geometry generation, raycasting, helpers, materials, lights, textures, and cameras are useful without GPU rendering.

## Does the loader upload textures to the GPU?

No. Loader output remains CPU data. Register assets with `Renderer` explicitly.

## Does Animato come by default?

No. Enable `animato` when you need the bridge. The bridge uses Animato 1.4.0.

## Can the website deploy to static hosting?

Yes. It is Leptos CSR and builds with Trunk for GitHub Pages at `/scenix/`.
