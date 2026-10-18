use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt;

use crate::node::{FogStyle, LightingStyle, NodeInput, ShaderNodeData, ShaderNodeKind, Stage};
use crate::value::{NodeValue, NodeValueKind};

pub use crate::node::{ShaderNodeId, ToneMapKind, UniformInput, VertexAttribute};

/// Maximum nodes accepted per live subgraph.
pub const NODE_GRAPH_LIMIT: usize = 1024;

/// Stable FNV-1a hash used for graph and shader identity.
#[inline]
pub fn fnv1a_hash(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// Validation or lowering diagnostic reported by graphs and backends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeDiagnostic {
    /// Node the diagnostic points at, when known.
    pub node: Option<ShaderNodeId>,
    /// Diagnostic family.
    pub kind: NodeDiagnosticKind,
}

impl NodeDiagnostic {
    /// Creates a diagnostic pointing at a node.
    #[inline]
    pub fn at_node(kind: NodeDiagnosticKind, node: ShaderNodeId) -> Self {
        Self {
            node: Some(node),
            kind,
        }
    }

    /// Creates a graph-level diagnostic.
    #[inline]
    pub fn graph_level(kind: NodeDiagnosticKind) -> Self {
        Self { node: None, kind }
    }
}

impl fmt::Display for NodeDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.node {
            Some(node) => write!(f, "{} at {node}", self.kind),
            None => fmt::Display::fmt(&self.kind, f),
        }
    }
}

impl Error for NodeDiagnostic {}

/// Diagnostic families produced by validation and backends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeDiagnosticKind {
    /// A port references an unknown node id.
    UnknownNode(ShaderNodeId),
    /// The live subgraph contains a reference cycle through this node.
    Cycle(ShaderNodeId),
    /// A required input port is unset.
    MissingInput { node: ShaderNodeId, slot: usize },
    /// An input port has the wrong value type.
    TypeMismatch {
        node: ShaderNodeId,
        slot: usize,
        expected: NodeValueKind,
        found: NodeValueKind,
    },
    /// An input must be a vector (`vec2`/`vec3`/`vec4`).
    VectorExpected {
        node: ShaderNodeId,
        slot: usize,
        found: NodeValueKind,
    },
    /// The node kind is not available in the requested stage.
    StageUnsupported {
        node: ShaderNodeId,
        found: &'static str,
    },
    /// The live subgraph exceeds [`NODE_GRAPH_LIMIT`].
    GraphTooLarge { max: usize },
    /// The fragment color output port is unset.
    MissingFragmentOutput,
    /// An output port has an unsupported type.
    OutputTypeMismatch {
        found: &'static str,
        expected: &'static str,
    },
    /// The `Component` selector is out of range for the input type.
    ComponentOutOfRange {
        node: ShaderNodeId,
        index: u8,
        max: u8,
    },
    /// A node keys material texture slots other than the built-in slot 0.
    UnsupportedTextureSlot { node: ShaderNodeId },
    /// A lighting style carries invalid band parameters.
    InvalidLightingStyle { node: ShaderNodeId },
    /// A fog style carries invalid parameters.
    InvalidFogStyle { node: ShaderNodeId },
}

impl fmt::Display for NodeDiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(id) => write!(f, "references unknown {id}"),
            Self::Cycle(id) => write!(f, "cycle through {id}"),
            Self::MissingInput { node, slot } => write!(f, "{node} misses input {slot}"),
            Self::TypeMismatch {
                slot,
                expected,
                found,
                ..
            } => write!(f, "input {slot} expects {expected}, found {found}"),
            Self::VectorExpected { slot, found, .. } => {
                write!(f, "input {slot} must be a vector, found {found}")
            }
            Self::StageUnsupported { found, .. } => {
                write!(f, "op `{found}` is not available in this stage")
            }
            Self::GraphTooLarge { max } => write!(f, "live subgraph exceeds {max} nodes"),
            Self::MissingFragmentOutput => write!(f, "fragment color output is unset"),
            Self::OutputTypeMismatch { found, expected } => {
                write!(f, "output is {found}, expected {expected}")
            }
            Self::ComponentOutOfRange { index, max, .. } => {
                write!(f, "component {index} out of range for {max} channels")
            }
            Self::UnsupportedTextureSlot { .. } => {
                write!(f, "v1.6 binds only material texture slot 0")
            }
            Self::InvalidLightingStyle { .. } => write!(f, "lighting style parameter invalid"),
            Self::InvalidFogStyle { .. } => write!(f, "fog style parameters invalid"),
        }
    }
}

impl From<NodeDiagnostic> for scenix_core::ValidationError {
    fn from(_diagnostic: NodeDiagnostic) -> Self {
        scenix_core::ValidationError::InvalidState
    }
}

/// One resolved port mismatch before the owning node id is attached.
enum PortDiagnostic {
    TypeMismatch {
        slot: usize,
        expected: NodeValueKind,
        found: NodeValueKind,
    },
    VectorExpected {
        slot: usize,
        found: NodeValueKind,
    },
    ComponentOutOfRange {
        index: u8,
        max: u8,
    },
}

/// Human-readable type label for diagnostics.
pub(crate) fn type_label(kind: NodeValueKind) -> &'static str {
    kind.wgsl_type()
}

/// Node DFS states used during validation.
enum Visit {
    Discovered,
    Finished,
}

/// A typed shader node graph: a validated DAG lowered by deterministic
/// backends.
///
/// Output ports live on the graph itself — `fragment_color`, optional
/// `fragment_alpha`, and optional `vertex_offset` — instead of dedicated
/// output nodes. Only the live subgraph reachable from the output ports is
/// validated and lowered; unreachable nodes are dead code and pruned.
///
/// # Example
/// ```
/// use scenix_nodes::{LightingStyle, NodeValue, ShaderGraph, ShaderNodeKind, UniformInput};
///
/// let mut graph = ShaderGraph::new();
/// let base_color = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
/// let shaded = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), base_color);
/// graph.add_fragment_output(shaded, None);
/// assert!(graph.validate(scenix_nodes::Stage::Fragment).is_empty());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShaderGraph {
    nodes: BTreeMap<ShaderNodeId, ShaderNodeData>,
    next_id: u32,
    fragment_color: Option<NodeInput>,
    fragment_alpha: Option<NodeInput>,
    vertex_offset: Option<NodeInput>,
}

impl ShaderGraph {
    /// Creates an empty graph.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the stored node count (including pruned dead nodes).
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the graph holds no nodes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the node at `id`.
    #[inline]
    pub fn get(&self, id: ShaderNodeId) -> Option<&ShaderNodeData> {
        self.nodes.get(&id)
    }

    /// Returns whether the graph holds `id`.
    #[inline]
    pub fn contains(&self, id: ShaderNodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Returns the fragment color output port.
    #[inline]
    pub fn fragment_color(&self) -> Option<&NodeInput> {
        self.fragment_color.as_ref()
    }

    /// Returns the fragment alpha output port, if authored.
    #[inline]
    pub fn fragment_alpha(&self) -> Option<&NodeInput> {
        self.fragment_alpha.as_ref()
    }

    /// Returns the optional vertex displacement port.
    #[inline]
    pub fn vertex_offset(&self) -> Option<&NodeInput> {
        self.vertex_offset.as_ref()
    }

    /// Allocates a fresh node handle and advances the cursor. Handles are
    /// never reused within a graph's lifetime.
    fn allocate(&mut self) -> ShaderNodeId {
        let id = ShaderNodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Adds a node from a resolved kind and input port list. Graph handles
    /// are never reused within a graph's lifetime.
    pub fn add(&mut self, kind: ShaderNodeKind, inputs: Vec<NodeInput>) -> ShaderNodeId {
        let id = self.allocate();
        self.nodes.insert(id, ShaderNodeData::new(id, kind, inputs));
        id
    }

    /// Adds a zero-input node.
    #[inline]
    pub fn push(&mut self, kind: ShaderNodeKind) -> ShaderNodeId {
        self.add(kind, Vec::new())
    }

    /// Adds a node with a single inline constant input.
    #[inline]
    pub fn constant(&mut self, value: NodeValue) -> ShaderNodeId {
        self.push(ShaderNodeKind::Constant(value))
    }

    /// Adds a node with one resolved input (`node` or inline value).
    #[inline]
    pub fn chain(&mut self, kind: ShaderNodeKind, input: impl Into<NodeInput>) -> ShaderNodeId {
        self.add(kind, alloc::vec![input.into()])
    }

    /// Removes every node and resets all output ports.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.next_id = 0;
        self.fragment_color = None;
        self.fragment_alpha = None;
        self.vertex_offset = None;
    }

    /// Sets the fragment color output, and optionally the alpha output.
    #[inline]
    pub fn add_fragment_output(&mut self, color: impl Into<NodeInput>, alpha: Option<NodeInput>) {
        self.fragment_color = Some(color.into());
        self.fragment_alpha = alpha;
    }

    /// Sets or clears the vertex displacement port (local-space `Vec3`).
    #[inline]
    pub fn set_vertex_offset(&mut self, offset: Option<ShaderNodeId>) {
        self.vertex_offset = offset.map(NodeInput::Node);
    }

    /// Replaces the input list of `id` (editor-style rewiring).
    ///
    /// # Errors
    /// Returns `UnknownNode` when the node is not part of the graph.
    pub fn set_node_inputs(
        &mut self,
        id: ShaderNodeId,
        inputs: Vec<NodeInput>,
    ) -> Result<(), NodeDiagnostic> {
        if !self.nodes.contains_key(&id) {
            return Err(NodeDiagnostic::graph_level(
                NodeDiagnosticKind::UnknownNode(id),
            ));
        }
        if let Some(node) = self.nodes.get_mut(&id) {
            node.inputs = inputs;
        }
        Ok(())
    }

    /// Collects every stored node id in ascending order.
    #[inline]
    pub fn ids(&self) -> impl Iterator<Item = ShaderNodeId> + '_ {
        self.nodes.keys().copied()
    }

    /// Returns the stable FNV-1a identity of the graph contents.
    pub fn shader_hash(&self) -> u64 {
        let mut payload = Vec::new();
        for node in self.nodes.values() {
            encode_node(node, &mut payload);
        }
        encode_port(self.fragment_color.as_ref(), &mut payload);
        encode_port(self.fragment_alpha.as_ref(), &mut payload);
        encode_port(self.vertex_offset.as_ref(), &mut payload);
        fnv1a_hash(&payload)
    }

    /// Returns the node handles referenced by the three output ports.
    fn port_roots(&self) -> Vec<ShaderNodeId> {
        let mut roots = Vec::with_capacity(3);
        for port in [
            self.fragment_color.as_ref(),
            self.fragment_alpha.as_ref(),
            self.vertex_offset.as_ref(),
        ] {
            if let Some(NodeInput::Node(id)) = port
                && self.nodes.contains_key(id)
            {
                roots.push(*id);
            }
        }
        roots
    }

    /// Validates the live subgraph for `stage`.
    ///
    /// Returns every diagnostic found; an empty result means the graph is
    /// ready to lower. Validation is O(live nodes) per call.
    pub fn validate(&self, stage: Stage) -> Vec<NodeDiagnostic> {
        let mut diagnostics = Vec::new();
        match self.fragment_color.as_ref() {
            None => {
                diagnostics.push(NodeDiagnostic::graph_level(
                    NodeDiagnosticKind::MissingFragmentOutput,
                ));
                return diagnostics;
            }
            Some(NodeInput::Value(value)) => {
                if !matches!(value.kind(), NodeValueKind::Vec3 | NodeValueKind::Vec4) {
                    diagnostics.push(NodeDiagnostic::graph_level(
                        NodeDiagnosticKind::OutputTypeMismatch {
                            found: type_label(value.kind()),
                            expected: "vec3<f32> or vec4<f32>",
                        },
                    ));
                }
            }
            Some(NodeInput::Node(id)) => {
                if !self.nodes.contains_key(id) {
                    diagnostics.push(NodeDiagnostic::graph_level(
                        NodeDiagnosticKind::UnknownNode(*id),
                    ));
                }
            }
        }

        if let Some(NodeInput::Value(value)) = self.fragment_alpha.as_ref()
            && value.kind() != NodeValueKind::Float
        {
            diagnostics.push(NodeDiagnostic::graph_level(
                NodeDiagnosticKind::OutputTypeMismatch {
                    found: type_label(value.kind()),
                    expected: "f32 alpha",
                },
            ));
        }
        if let Some(NodeInput::Node(id)) = self.vertex_offset.as_ref()
            && !self.nodes.contains_key(id)
        {
            diagnostics.push(NodeDiagnostic::graph_level(
                NodeDiagnosticKind::UnknownNode(*id),
            ));
        }

        // `Some(kind)` resolves a node; `None` poisons it so ancestors stop
        // cascading errors that were already reported here.
        let mut resolved: BTreeMap<ShaderNodeId, Option<NodeValueKind>> = BTreeMap::new();
        let mut visits: BTreeMap<ShaderNodeId, Visit> = BTreeMap::new();

        for root in self.port_roots() {
            if let alloc::collections::btree_map::Entry::Vacant(entry) = visits.entry(root) {
                entry.insert(Visit::Discovered);
                self.walk_and_check(root, stage, &mut resolved, &mut visits, &mut diagnostics);
            }
        }

        if resolved.len() > NODE_GRAPH_LIMIT {
            diagnostics.push(NodeDiagnostic::graph_level(
                NodeDiagnosticKind::GraphTooLarge {
                    max: NODE_GRAPH_LIMIT,
                },
            ));
        }

        let check_port = |port: Option<&NodeInput>,
                          expected: &'static str,
                          accept: &dyn Fn(NodeValueKind) -> bool,
                          diagnostics: &mut Vec<NodeDiagnostic>| {
            if let Some(NodeInput::Node(id)) = port
                && let Some(Some(kind)) = resolved.get(id)
                && !accept(*kind)
            {
                diagnostics.push(NodeDiagnostic::graph_level(
                    NodeDiagnosticKind::OutputTypeMismatch {
                        found: type_label(*kind),
                        expected,
                    },
                ));
            }
        };
        check_port(
            self.fragment_color.as_ref(),
            "vec3<f32> or vec4<f32>",
            &|kind| matches!(kind, NodeValueKind::Vec3 | NodeValueKind::Vec4),
            &mut diagnostics,
        );
        check_port(
            self.fragment_alpha.as_ref(),
            "f32 alpha",
            &|kind| kind == NodeValueKind::Float,
            &mut diagnostics,
        );
        check_port(
            self.vertex_offset.as_ref(),
            "vec3<f32> vertex offset",
            &|kind| kind == NodeValueKind::Vec3,
            &mut diagnostics,
        );

        diagnostics
    }

    /// Iterative post-order DFS that type checks every reachable node.
    fn walk_and_check(
        &self,
        root: ShaderNodeId,
        stage: Stage,
        resolved: &mut BTreeMap<ShaderNodeId, Option<NodeValueKind>>,
        visits: &mut BTreeMap<ShaderNodeId, Visit>,
        diagnostics: &mut Vec<NodeDiagnostic>,
    ) {
        let limit_hit = core::cell::Cell::new(false);
        let mut frames = alloc::vec![(root, 0_usize)];

        while let Some((id, cursor)) = frames.pop() {
            if resolved.len() > NODE_GRAPH_LIMIT {
                if !limit_hit.get() {
                    limit_hit.set(true);
                    diagnostics.push(NodeDiagnostic::graph_level(
                        NodeDiagnosticKind::GraphTooLarge {
                            max: NODE_GRAPH_LIMIT,
                        },
                    ));
                }
                return;
            }

            let mut descend = None;
            let node = self.nodes.get(&id);
            if let Some(node) = node {
                let mut cursor = cursor;
                while cursor < node.inputs.len() {
                    match &node.inputs[cursor] {
                        NodeInput::Value(_) => cursor += 1,
                        NodeInput::Node(child) => {
                            if !self.nodes.contains_key(child) {
                                diagnostics.push(NodeDiagnostic::at_node(
                                    NodeDiagnosticKind::UnknownNode(*child),
                                    id,
                                ));
                                cursor += 1;
                            } else {
                                match visits.get(child) {
                                    Some(Visit::Finished) => cursor += 1,
                                    Some(Visit::Discovered) => {
                                        diagnostics.push(NodeDiagnostic::at_node(
                                            NodeDiagnosticKind::Cycle(*child),
                                            *child,
                                        ));
                                        cursor += 1;
                                    }
                                    None => {
                                        visits.insert(*child, Visit::Discovered);
                                        descend = Some(*child);
                                        cursor += 1;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(child) = descend {
                    frames.push((id, cursor));
                    frames.push((child, 0));
                    continue;
                }
            }

            if let Some(node) = node {
                let _ = visits.insert(id, Visit::Finished);
                self.type_check_node(node, stage, resolved, diagnostics);
            }
        }
    }

    /// Type checks and records the output kind of one node.
    fn type_check_node(
        &self,
        node: &ShaderNodeData,
        stage: Stage,
        resolved: &mut BTreeMap<ShaderNodeId, Option<NodeValueKind>>,
        diagnostics: &mut Vec<NodeDiagnostic>,
    ) {
        let id = node.id;
        let kind = &node.kind;

        if !kind.stage_kind().allows(stage) {
            diagnostics.push(NodeDiagnostic::at_node(
                NodeDiagnosticKind::StageUnsupported {
                    node: id,
                    found: kind.identifier(),
                },
                id,
            ));
            resolved.insert(id, None);
            return;
        }
        if let ShaderNodeKind::Lighting(style) = kind
            && !style.is_valid()
        {
            diagnostics.push(NodeDiagnostic::at_node(
                NodeDiagnosticKind::InvalidLightingStyle { node: id },
                id,
            ));
            resolved.insert(id, None);
            return;
        }
        if let ShaderNodeKind::Fog(style) = kind {
            let color_kind = match style {
                FogStyle::Exponential { color, .. } | FogStyle::Linear { color, .. } => {
                    color.kind()
                }
            };
            if !matches!(color_kind, NodeValueKind::Vec3) {
                diagnostics.push(NodeDiagnostic::at_node(
                    NodeDiagnosticKind::InvalidFogStyle { node: id },
                    id,
                ));
                resolved.insert(id, None);
                return;
            }
        }

        if node.inputs.len() < kind.port_count() {
            let slot = node.inputs.len();
            diagnostics.push(NodeDiagnostic::at_node(
                NodeDiagnosticKind::MissingInput { node: id, slot },
                id,
            ));
            resolved.insert(id, None);
            return;
        }

        let mut kinds = Vec::with_capacity(kind.port_count());
        for slot in 0..kind.port_count() {
            let found = match &node.inputs[slot] {
                NodeInput::Value(value) => Some(value.kind()),
                NodeInput::Node(child) => resolved.get(child).copied().flatten(),
            };
            let Some(found) = found else {
                // Unknown or poisoned input: already reported upstream.
                resolved.insert(id, None);
                return;
            };
            if let Some(expected) = kind.input_expectation(slot)
                && found != expected
            {
                diagnostics.push(NodeDiagnostic::at_node(
                    NodeDiagnosticKind::TypeMismatch {
                        node: id,
                        slot,
                        expected,
                        found,
                    },
                    id,
                ));
                resolved.insert(id, None);
                return;
            }
            kinds.push(found);
        }

        if let Some(port) = check_ports(kind, &kinds) {
            let diagnostic = match port {
                PortDiagnostic::TypeMismatch {
                    slot,
                    expected,
                    found,
                } => NodeDiagnosticKind::TypeMismatch {
                    node: id,
                    slot,
                    expected,
                    found,
                },
                PortDiagnostic::VectorExpected { slot, found } => {
                    NodeDiagnosticKind::VectorExpected {
                        node: id,
                        slot,
                        found,
                    }
                }
                PortDiagnostic::ComponentOutOfRange { index, max } => {
                    NodeDiagnosticKind::ComponentOutOfRange {
                        node: id,
                        index,
                        max,
                    }
                }
            };
            diagnostics.push(NodeDiagnostic::at_node(diagnostic, id));
            resolved.insert(id, None);
            return;
        }

        resolved.insert(id, Some(kind.output_kind(&kinds)));
    }

    /// Deterministic post-order emission list of live nodes (leaves first),
    /// reachable from every output port. This is the exact order backends
    /// lower statements in; shared subexpressions appear once.
    pub fn evaluation_order(&self) -> Vec<ShaderNodeId> {
        self.walk_order(&self.port_roots())
    }

    /// Post-order emission order of the fragment-reachable live subgraph.
    pub fn fragment_order(&self) -> Vec<ShaderNodeId> {
        let mut roots = Vec::with_capacity(2);
        for port in [self.fragment_color.as_ref(), self.fragment_alpha.as_ref()] {
            if let Some(NodeInput::Node(id)) = port
                && self.nodes.contains_key(id)
            {
                roots.push(*id);
            }
        }
        self.walk_order(&roots)
    }

    /// Post-order emission order of the optional vertex-displacement
    /// subgraph.
    pub fn vertex_order(&self) -> Option<Vec<ShaderNodeId>> {
        match self.vertex_offset.as_ref() {
            Some(NodeInput::Node(id)) if self.nodes.contains_key(id) => {
                Some(self.walk_order(&[*id]))
            }
            _ => None,
        }
    }

    /// Shared post-order walker over the given roots.
    fn walk_order(&self, roots: &[ShaderNodeId]) -> Vec<ShaderNodeId> {
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut live: BTreeMap<ShaderNodeId, ()> = BTreeMap::new();
        for root in roots {
            collect_live(self, *root, &mut live);
        }

        let mut done: BTreeMap<ShaderNodeId, ()> = BTreeMap::new();
        for root in roots {
            if done.contains_key(root) {
                continue;
            }
            done.insert(*root, ());
            let mut frames = alloc::vec![(*root, 0_usize)];
            while let Some((id, cursor)) = frames.pop() {
                let mut descend = None;
                if let Some(node) = self.nodes.get(&id) {
                    let mut cursor = cursor;
                    while cursor < node.inputs.len() {
                        if let NodeInput::Node(child) = &node.inputs[cursor]
                            && live.contains_key(child)
                            && !done.contains_key(child)
                        {
                            done.insert(*child, ());
                            descend = Some(*child);
                            break;
                        }
                        cursor += 1;
                    }
                }
                if let Some(child) = descend {
                    frames.push((id, cursor + 1));
                    frames.push((child, 0));
                    continue;
                }
                if self.nodes.contains_key(&id) {
                    order.push(id);
                }
            }
        }
        order
    }

    /// Resolves live-node output kinds after successful validation.
    ///
    /// Returns the map used by code backends, or the first failing
    /// diagnostic when validation did not pass.
    pub fn resolve(
        &self,
        stage: Stage,
    ) -> Result<BTreeMap<ShaderNodeId, Option<NodeValueKind>>, NodeDiagnostic> {
        let diagnostics = self.validate(stage);
        if let Some(first) = diagnostics.first() {
            return Err(first.clone());
        }

        let mut resolved: BTreeMap<ShaderNodeId, Option<NodeValueKind>> = BTreeMap::new();
        let mut visits: BTreeMap<ShaderNodeId, Visit> = BTreeMap::new();
        let mut sink = Vec::new();
        for root in self.port_roots() {
            if let alloc::collections::btree_map::Entry::Vacant(entry) = visits.entry(root) {
                entry.insert(Visit::Discovered);
                self.walk_and_check(root, stage, &mut resolved, &mut visits, &mut sink);
            }
        }
        Ok(resolved)
    }
}

/// Depth-first live-set collector.
fn collect_live(graph: &ShaderGraph, root: ShaderNodeId, live: &mut BTreeMap<ShaderNodeId, ()>) {
    let mut frames = alloc::vec![root];
    while let Some(id) = frames.pop() {
        if live.contains_key(&id) {
            continue;
        }
        let Some(node) = graph.get(id) else {
            continue;
        };
        let _ = live.insert(id, ());
        for input in &node.inputs {
            if let NodeInput::Node(child) = input
                && graph.contains(*child)
            {
                frames.push(*child);
            }
        }
    }
}

/// Cross-port type rules for flexibly typed kinds.
fn check_ports(kind: &ShaderNodeKind, kinds: &[NodeValueKind]) -> Option<PortDiagnostic> {
    /// Accepts matching vector types, or broadcasts a float operand.
    fn float_or_match(a: NodeValueKind, b: NodeValueKind) -> bool {
        a == b || a.is_float() || b.is_float()
    }

    /// Whether a kind is any vector shape.
    fn is_vector(kind: NodeValueKind) -> bool {
        !kind.is_float()
    }

    // Color-like node inputs accept rgb or rgba colors; alpha is dropped
    // during code generation.
    if matches!(
        kind,
        ShaderNodeKind::Lighting(_)
            | ShaderNodeKind::ToneMap(_)
            | ShaderNodeKind::Fog(_)
            | ShaderNodeKind::SrgbToLinear
            | ShaderNodeKind::LinearToSrgb
    ) && kinds.first().is_some_and(|found| !is_vector(*found))
    {
        return Some(PortDiagnostic::VectorExpected {
            slot: 0,
            found: kinds[0],
        });
    }

    match kind {
        ShaderNodeKind::Add
        | ShaderNodeKind::Subtract
        | ShaderNodeKind::Multiply
        | ShaderNodeKind::Divide
        | ShaderNodeKind::Min
        | ShaderNodeKind::Max => {
            if !float_or_match(kinds[0], kinds[1]) {
                return Some(PortDiagnostic::TypeMismatch {
                    slot: 1,
                    expected: kinds[0],
                    found: kinds[1],
                });
            }
            None
        }
        ShaderNodeKind::Dot => {
            if kinds[0] != kinds[1] || !is_vector(kinds[0]) {
                return Some(PortDiagnostic::TypeMismatch {
                    slot: 1,
                    expected: kinds[0],
                    found: kinds[1],
                });
            }
            None
        }
        ShaderNodeKind::Cross => {
            if kinds != [NodeValueKind::Vec3, NodeValueKind::Vec3] {
                return Some(PortDiagnostic::TypeMismatch {
                    slot: 1,
                    expected: NodeValueKind::Vec3,
                    found: kinds[1],
                });
            }
            None
        }
        ShaderNodeKind::Length | ShaderNodeKind::Normalize => {
            if !is_vector(kinds[0]) {
                return Some(PortDiagnostic::VectorExpected {
                    slot: 0,
                    found: kinds[0],
                });
            }
            None
        }
        ShaderNodeKind::Mix | ShaderNodeKind::Clamp => {
            if kinds[0] != kinds[1] {
                return Some(PortDiagnostic::TypeMismatch {
                    slot: 1,
                    expected: kinds[0],
                    found: kinds[1],
                });
            }
            None
        }
        ShaderNodeKind::Splat(_) => {
            if kinds[0] != NodeValueKind::Float {
                return Some(PortDiagnostic::TypeMismatch {
                    slot: 0,
                    expected: NodeValueKind::Float,
                    found: kinds[0],
                });
            }
            None
        }
        ShaderNodeKind::Component(index) => {
            if !is_vector(kinds[0]) {
                return Some(PortDiagnostic::VectorExpected {
                    slot: 0,
                    found: kinds[0],
                });
            }
            let max = kinds[0].components() as u8;
            if index.index() >= max {
                return Some(PortDiagnostic::ComponentOutOfRange {
                    index: index.index(),
                    max,
                });
            }
            None
        }
        // Color-like inputs take vec3 or vec4 (alpha ignored for vec4).
        ShaderNodeKind::Lighting(_)
        | ShaderNodeKind::ToneMap(_)
        | ShaderNodeKind::Fog(_)
        | ShaderNodeKind::SrgbToLinear
        | ShaderNodeKind::LinearToSrgb => {
            if !matches!(kinds[0], NodeValueKind::Vec3 | NodeValueKind::Vec4) {
                return Some(PortDiagnostic::TypeMismatch {
                    slot: 0,
                    expected: NodeValueKind::Vec3,
                    found: kinds[0],
                });
            }
            None
        }
        _ => None,
    }
}

/// Encodes a node deterministically for hashing.
fn encode_node(node: &ShaderNodeData, payload: &mut Vec<u8>) {
    payload.extend_from_slice(&node.id.0.to_le_bytes());
    encode_kind_payload(&node.kind, payload);
    for input in &node.inputs {
        encode_port(Some(input), payload);
    }
}

/// Encodes kind parameters (node id and input ports excluded) so backends
/// can build identical CSE keys.
pub(crate) fn encode_kind_payload(kind: &ShaderNodeKind, payload: &mut Vec<u8>) {
    payload.extend_from_slice(kind.identifier().as_bytes());
    match kind {
        ShaderNodeKind::Constant(value) => encode_value(value, payload),
        ShaderNodeKind::Uniform(uniform) => {
            payload.extend_from_slice(uniform.identifier().as_bytes());
        }
        ShaderNodeKind::Attribute(attribute) => {
            payload.extend_from_slice(attribute.varying_name().as_bytes());
        }
        ShaderNodeKind::Splat(target) => payload.push(kind_tag(*target)),
        ShaderNodeKind::Component(index) => payload.push(index.index()),
        ShaderNodeKind::Fresnel(power) => {
            payload.extend_from_slice(&power.to_bits().to_le_bytes());
        }
        ShaderNodeKind::Lighting(style) => match style {
            LightingStyle::Lambert => payload.push(0),
            LightingStyle::Phong => payload.push(1),
            LightingStyle::Toon(bands) => payload.extend_from_slice(&bands.to_le_bytes()),
        },
        ShaderNodeKind::ToneMap(tonemap) => match tonemap {
            crate::node::ToneMapKind::None => payload.push(0),
            crate::node::ToneMapKind::Reinhard => payload.push(1),
            crate::node::ToneMapKind::Aces => payload.push(2),
            crate::node::ToneMapKind::Exposure(exposure) => {
                payload.push(3);
                payload.extend_from_slice(&exposure.to_bits().to_le_bytes());
            }
        },
        ShaderNodeKind::Fog(style) => match style {
            FogStyle::Exponential { density, color } => {
                payload.extend_from_slice(&density.to_bits().to_le_bytes());
                encode_value(color, payload);
            }
            FogStyle::Linear { near, far, color } => {
                payload.extend_from_slice(&near.to_bits().to_le_bytes());
                payload.extend_from_slice(&far.to_bits().to_le_bytes());
                encode_value(color, payload);
            }
        },
        ShaderNodeKind::PostParam(index) => payload.push(*index),
        _ => {}
    }
}

/// Encodes an inline value deterministically for hashing.
fn encode_value(value: &NodeValue, payload: &mut Vec<u8>) {
    for word in value.bit_pattern() {
        payload.extend_from_slice(&word.to_le_bytes());
    }
}

/// Encodes an output port deterministically for hashing.
fn encode_port(port: Option<&NodeInput>, payload: &mut Vec<u8>) {
    match port {
        Some(NodeInput::Node(id)) => {
            payload.push(0);
            payload.extend_from_slice(&id.0.to_le_bytes());
        }
        Some(NodeInput::Value(value)) => {
            payload.push(1);
            encode_value(value, payload);
        }
        None => payload.push(2),
    }
}

/// One-byte tag for a value kind.
#[inline]
const fn kind_tag(kind: NodeValueKind) -> u8 {
    match kind {
        NodeValueKind::Float => 0,
        NodeValueKind::Vec2 => 1,
        NodeValueKind::Vec3 => 2,
        NodeValueKind::Vec4 => 3,
    }
}
