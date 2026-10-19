use scenix::{
    EnvironmentMaterial, Heightmap, LodBand, LodPolicy, MaterialId, PerspectiveCamera, Renderer,
    RendererConfig, SceneGraph, SceneNode, SkyId, SkyMaterial, TerrainConfig, TerrainGrid,
    TerrainId, TerrainMaterial, Transform, Vec3, WaterId, WaterMaterial, WaveField,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(320, 240)).await?;

        // Terrain: a small two-chunk heightfield registered at LOD 0.
        let map = Heightmap::from_fn(17, 17, 1.0, |ix, iz| ((ix + iz).min(12) as f32) * 0.5);
        let policy = LodPolicy::new(&[LodBand::new(40.0, 0)]).expect("valid policy");
        let grid = TerrainGrid::generate(&map, TerrainConfig::new(2, 8, policy));
        let chunk = grid.chunk_geometry(0, 0).expect("chunk geometry");
        let terrain_material = TerrainMaterial::default_palette();
        renderer.register_environment(
            MaterialId::new(1),
            EnvironmentMaterial::Terrain(&terrain_material, &chunk),
        )?;

        // Sky: a procedural dominating dome (no textures needed).
        let sky_material = SkyMaterial::clear_day();
        renderer
            .register_environment(MaterialId::new(2), EnvironmentMaterial::Sky(&sky_material))?;

        // Water: an animated patch with foam enabled (the renderer paves
        // the grid from the wave field at GPU upload time).
        let waves = WaveField::ocean_shore();
        let water_material = WaterMaterial::clear().with_foam(0.42, 5.0);
        renderer.register_environment(
            MaterialId::new(3),
            EnvironmentMaterial::Water(&water_material, &waves),
        )?;

        // Scene nodes carry the payload through the schema; the renderer
        // looks slots up by node material id.
        let mut scene = SceneGraph::new();
        scene.add(
            SceneNode::terrain("hills", TerrainId::new(1), MaterialId::new(1)).transform(
                Transform::new(
                    Vec3::new(-4.0, 0.0, -4.0),
                    scenix::Quat::IDENTITY,
                    Vec3::new(1.0, 1.0, 1.0),
                ),
            ),
        );
        scene.add(
            SceneNode::sky("backdrop", SkyId::new(1), MaterialId::new(2)).transform(
                Transform::new(
                    Vec3::ZERO,
                    scenix::Quat::IDENTITY,
                    Vec3::new(120.0, 120.0, 120.0),
                ),
            ),
        );
        let lake = scene.add(
            SceneNode::water("lake", WaterId::new(1), MaterialId::new(3)).transform(
                Transform::new(
                    Vec3::new(0.0, 0.25, 3.0),
                    scenix::Quat::IDENTITY,
                    Vec3::new(1.0, 1.0, 1.0),
                ),
            ),
        );
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(55.0, 320.0 / 240.0, 0.1, 400.0)
            .position(Vec3::new(0.0, 3.0, 9.0))
            .target(Vec3::new(0.0, 1.5, 0.0));
        let stats = renderer.render(&scene, &camera)?;
        println!(
            "environment frame: {} slots registered, {} visible in frustum",
            stats.scene_environments, stats.visible_environments
        );
        println!(
            "scene: {} meshes total, {} visible",
            stats.scene_meshes, stats.visible_meshes
        );
        let [blue, _green, _red, _alpha] = renderer.read_headless_pixel()?;
        println!("backdrop blue channel: {blue}");

        // Unregistering the lake keeps the terrain and sky drawing.
        assert!(renderer.unregister_environment(MaterialId::new(3)));
        scene.remove(lake)?;
        scene.update_world_transforms();
        let stats = renderer.render(&scene, &camera)?;
        println!(
            "after removing a slot: {} visible environments",
            stats.visible_environments
        );

        Ok(())
    })
}
