//! GPU-gated node-material render tests (scenix v1.6.0).
//!
//! Run with `SCENIX_RUN_GPU_TESTS=1`; uses lavapipe/headless Vulkan in CI.

use scenix_camera::PerspectiveCamera;
use scenix_core::{Color, LightId, MaterialId, MeshId, ScenixError};
use scenix_light::AmbientLight;
use scenix_math::Transform;
use scenix_math::Vec3;
use scenix_mesh::box_geometry;
use scenix_nodes::{
    LightingStyle, NodeMaterial, NodeValueKind, ShaderGraph, ShaderNodeKind, Stage, UniformInput,
};
use scenix_renderer::{Renderer, RendererConfig};
use scenix_scene::{SceneGraph, SceneNode};

fn run_gpu_tests() -> bool {
    std::env::var("SCENIX_RUN_GPU_TESTS").as_deref() == Ok("1")
}

/// A node material colored by the base-color uniform (red) through a
/// Lambert lighting term.
fn red_node_material() -> NodeMaterial {
    let mut graph = ShaderGraph::new();
    let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
    let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), base);
    graph.add_fragment_output(lit, None);
    NodeMaterial::new("node-red", graph).with_base_color(Color::from_rgb(0.9, 0.15, 0.05))
}

#[test]
fn node_material_draw_reaches_the_headless_framebuffer() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;
        let mesh_id = MeshId::new(1);
        let node_id = MaterialId::new(9);

        renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
        renderer.register_node_material(node_id, &red_node_material())?;
        renderer.register_ambient_light(
            LightId::new(1),
            AmbientLight::new(Color::from_rgb(0.6, 0.6, 0.6), 1.0),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("cube", mesh_id, node_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 4.0))
            .target(Vec3::ZERO);

        let stats = renderer.render(&scene, &camera)?;
        assert_eq!(stats.visible_meshes, 1);
        assert_eq!(stats.opaque_draws, 1);

        let pixel = renderer.read_headless_pixel()?;
        assert!(
            pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0,
            "node pipeline must shade the cube"
        );
        Ok(())
    })
}

#[test]
fn node_draws_share_the_frame_with_builtin_pipelines() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;
        let mesh_id = MeshId::new(1);
        let builtin_id = MaterialId::new(2);
        let node_id = MaterialId::new(9);

        renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
        renderer.register_unlit_material(
            builtin_id,
            &scenix_material::UnlitMaterial::new().color(Color::from_rgb(0.1, 0.5, 0.9)),
        )?;
        renderer.register_node_material(node_id, &red_node_material())?;
        renderer.register_ambient_light(
            LightId::new(1),
            AmbientLight::new(Color::from_rgb(0.7, 0.7, 0.7), 1.0),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(
            SceneNode::mesh("node-cube", mesh_id, node_id)
                .transform(Transform::from_translation(Vec3::new(-1.5, 0.0, 0.0))),
        );
        scene.add(
            SceneNode::mesh("builtin-cube", mesh_id, builtin_id)
                .transform(Transform::from_translation(Vec3::new(1.5, 0.0, 0.0))),
        );
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 5.0))
            .target(Vec3::ZERO);

        let stats = renderer.render(&scene, &camera)?;
        assert_eq!(stats.visible_meshes, 2);
        assert_eq!(stats.opaque_draws, 2);
        Ok(())
    })
}

#[test]
fn unknown_node_materials_reject_during_culling() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;
        let mesh_id = MeshId::new(1);
        let missing_id = MaterialId::new(42);

        renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("lost-cube", mesh_id, missing_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 4.0))
            .target(Vec3::ZERO);

        let result = renderer.render(&scene, &camera);
        assert!(result.is_err(), "unregistered materials are rejected");
        Ok(())
    })
}

#[test]
fn register_rejects_invalid_graphs_and_null_ids() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;

        // Valid: sample the uv attribute (no texture registered; the default
        // white texture binds instead).
        let mut graph = ShaderGraph::new();
        let uv = graph.push(ShaderNodeKind::Attribute(scenix_nodes::VertexAttribute::Uv));
        let sample = graph.chain(ShaderNodeKind::TextureSample, uv);
        graph.add_fragment_output(sample, None);
        let valid = NodeMaterial::new("valid", graph);
        renderer
            .register_node_material(MaterialId::new(3), &valid)
            .expect("valid graph registers");

        // Broken: ScreenUv (vec2) must not feed a scalar splat.
        let mut broken_graph = ShaderGraph::new();
        let screen_uv = broken_graph.push(ShaderNodeKind::ScreenUv);
        let flat = broken_graph.chain(ShaderNodeKind::Splat(NodeValueKind::Float), screen_uv);
        let lit = broken_graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), flat);
        broken_graph.add_fragment_output(lit, None);
        assert!(!broken_graph.validate(Stage::Fragment).is_empty());
        let broken = NodeMaterial::new("broken", broken_graph);
        assert!(
            renderer
                .register_node_material(MaterialId::new(4), &broken)
                .is_err()
        );

        // Null ids are rejected outright.
        assert!(
            renderer
                .register_node_material(MaterialId::new(0), &valid)
                .is_err()
        );
        Ok(())
    })
}
