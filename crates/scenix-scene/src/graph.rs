use alloc::vec::Vec;
use core::mem;

use scenix_core::{NodeId, ValidationError};
use scenix_math::{Mat4, Transform};
use slotmap::{SlotMap, new_key_type};

use crate::{BreadthFirstIter, DepthFirstIter, Fog, SceneNode};

new_key_type! {
    pub(crate) struct PrivateSceneKey;
}

struct NodeRecord {
    id: NodeId,
    node: SceneNode,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    world_matrix: Mat4,
    dirty: bool,
}

impl NodeRecord {
    fn new(node: SceneNode, parent: Option<NodeId>) -> Self {
        Self {
            id: NodeId::default(),
            node,
            parent,
            children: Vec::new(),
            world_matrix: Mat4::IDENTITY,
            dirty: true,
        }
    }
}

/// SlotMap-backed scene node hierarchy with cached world transforms.
pub struct SceneGraph {
    nodes: SlotMap<PrivateSceneKey, NodeRecord>,
    roots: Vec<NodeId>,
    id_to_key: Vec<Option<PrivateSceneKey>>,
    next_id: u64,
    dirty_roots: Vec<NodeId>,
    fog: Option<Fog>,
}

impl SceneGraph {
    /// Creates an empty scene graph.
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates an empty scene graph with storage reserved for `capacity` nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut id_to_key = Vec::with_capacity(capacity.saturating_add(1));
        id_to_key.push(None);
        Self {
            nodes: SlotMap::with_capacity_and_key(capacity),
            roots: Vec::new(),
            id_to_key,
            next_id: 1,
            dirty_roots: Vec::new(),
            fog: None,
        }
    }

    /// Adds a root node and returns its graph-local ID.
    pub fn add(&mut self, node: SceneNode) -> NodeId {
        let key = self.nodes.insert(NodeRecord::new(node, None));
        let id = self.allocate_id(key);
        self.nodes[key].id = id;
        self.roots.push(id);
        self.dirty_roots.push(id);
        id
    }

    /// Adds a child node under `parent`.
    pub fn add_child(
        &mut self,
        parent: NodeId,
        node: SceneNode,
    ) -> Result<NodeId, ValidationError> {
        let parent_key = self.key_or_err(parent)?;
        let key = self.nodes.insert(NodeRecord::new(node, Some(parent)));
        let id = self.allocate_id(key);
        self.nodes[key].id = id;
        self.nodes[parent_key].children.push(id);
        self.dirty_roots.push(id);
        Ok(id)
    }

    /// Removes `id` and its descendants.
    pub fn remove(&mut self, id: NodeId) -> Result<(), ValidationError> {
        self.key_or_err(id)?;
        let parent = self.get_record(id).and_then(|record| record.parent);
        if let Some(parent) = parent {
            if let Some(parent_key) = self.key(parent) {
                self.nodes[parent_key].children.retain(|child| *child != id);
            }
        } else {
            self.roots.retain(|root| *root != id);
        }

        let mut stack = Vec::from([id]);
        let mut removed = Vec::new();
        while let Some(current) = stack.pop() {
            let Some(key) = self.key(current) else {
                continue;
            };
            let Some(record) = self.nodes.get(key) else {
                continue;
            };
            stack.extend(record.children.iter().copied());
            removed.push((current, key));
        }

        for (removed_id, key) in &removed {
            self.nodes.remove(*key);
            self.clear_id(*removed_id);
        }
        self.dirty_roots
            .retain(|dirty| !removed.iter().any(|(removed_id, _)| removed_id == dirty));

        Ok(())
    }

    /// Reparents `node` under `new_parent`, or makes it a root when `None`.
    pub fn reparent(
        &mut self,
        node: NodeId,
        new_parent: Option<NodeId>,
    ) -> Result<(), ValidationError> {
        self.key_or_err(node)?;
        if let Some(parent) = new_parent {
            self.key_or_err(parent)?;
            if self.is_descendant(parent, node) {
                return Err(ValidationError::InvalidState);
            }
        }

        let old_parent = self.get_record(node).and_then(|record| record.parent);
        if old_parent == new_parent {
            return Ok(());
        }

        if let Some(parent) = old_parent {
            let parent_key = self.key_or_err(parent)?;
            self.nodes[parent_key]
                .children
                .retain(|child| *child != node);
        } else {
            self.roots.retain(|root| *root != node);
        }

        if let Some(parent) = new_parent {
            let parent_key = self.key_or_err(parent)?;
            self.nodes[parent_key].children.push(node);
        } else {
            self.roots.push(node);
        }

        let node_key = self.key_or_err(node)?;
        self.nodes[node_key].parent = new_parent;
        self.mark_subtree_dirty(node)?;
        Ok(())
    }

    /// Returns immutable node data for `id`.
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&SceneNode> {
        self.get_record(id).map(|record| &record.node)
    }

    /// Returns mutable node data for `id` and marks the subtree dirty.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.mark_subtree_dirty(id).ok()?;
        let key = self.key(id)?;
        self.nodes.get_mut(key).map(|record| &mut record.node)
    }

    /// Sets a node's local transform and marks its subtree dirty.
    pub fn set_local_transform(
        &mut self,
        id: NodeId,
        transform: Transform,
    ) -> Result<(), ValidationError> {
        let key = self.key_or_err(id)?;
        self.nodes[key].node.transform = transform;
        self.mark_subtree_dirty(id)
    }

    /// Returns the parent ID for `id`, if `id` is valid and not a root.
    #[inline]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.get_record(id).and_then(|record| record.parent)
    }

    /// Returns the child IDs for `id`.
    #[inline]
    pub fn children(&self, id: NodeId) -> Option<&[NodeId]> {
        self.get_record(id).map(|record| record.children.as_slice())
    }

    /// Returns root node IDs in insertion order.
    #[inline]
    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }

    /// Updates cached world transforms for dirty subtrees.
    pub fn update_world_transforms(&mut self) {
        let mut dirty_roots = mem::take(&mut self.dirty_roots);
        dirty_roots.sort_unstable();
        dirty_roots.dedup();
        for id in dirty_roots {
            if let Some(root) = self.highest_dirty_ancestor(id) {
                self.update_subtree(root);
            }
        }
    }

    /// Returns the cached world matrix for `id`.
    #[inline]
    pub fn world_matrix(&self, id: NodeId) -> Option<Mat4> {
        self.get_record(id).map(|record| record.world_matrix)
    }

    /// Returns the cached world transform for `id`, if it can be decomposed.
    #[inline]
    pub fn world_transform(&self, id: NodeId) -> Option<Transform> {
        self.world_matrix(id).and_then(Transform::from_mat4)
    }

    /// Finds the first node with `name` in depth-first order.
    pub fn find_by_name(&self, name: &str) -> Option<NodeId> {
        self.iter_depth_first()
            .find(|id| self.get(*id).is_some_and(|node| node.name == name))
    }

    /// Returns a depth-first node ID iterator.
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    #[inline]
    pub fn iter_depth_first(&self) -> DepthFirstIter<'_> {
        DepthFirstIter::new(self)
    }

    /// Returns a breadth-first node ID iterator.
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    #[inline]
    pub fn iter_breadth_first(&self) -> BreadthFirstIter<'_> {
        BreadthFirstIter::new(self)
    }

    /// Returns scene fog settings.
    #[inline]
    pub const fn fog(&self) -> Option<Fog> {
        self.fog
    }

    /// Sets scene fog settings.
    #[inline]
    pub fn set_fog(&mut self, fog: Option<Fog>) {
        self.fog = fog;
    }

    fn key(&self, id: NodeId) -> Option<PrivateSceneKey> {
        let index = id_index(id)?;
        self.id_to_key.get(index).copied().flatten()
    }

    fn key_or_err(&self, id: NodeId) -> Result<PrivateSceneKey, ValidationError> {
        self.key(id).ok_or(ValidationError::InvalidId)
    }

    fn get_record(&self, id: NodeId) -> Option<&NodeRecord> {
        self.key(id).and_then(|key| self.nodes.get(key))
    }

    fn allocate_id(&mut self, key: PrivateSceneKey) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("scenix scene graph exhausted node ids");
        let index = id_index(id).expect("scenix scene graph node id exceeded platform index size");
        if self.id_to_key.len() <= index {
            self.id_to_key.resize(index + 1, None);
        }
        self.id_to_key[index] = Some(key);
        id
    }

    fn clear_id(&mut self, id: NodeId) {
        if let Some(index) = id_index(id)
            && let Some(slot) = self.id_to_key.get_mut(index)
        {
            *slot = None;
        }
    }

    fn mark_subtree_dirty(&mut self, id: NodeId) -> Result<(), ValidationError> {
        let root_key = self.key_or_err(id)?;
        if self.nodes[root_key].dirty {
            return Ok(());
        }

        self.dirty_roots.push(id);
        let mut stack = Vec::from([id]);
        while let Some(current) = stack.pop() {
            let Some(key) = self.key(current) else {
                continue;
            };
            let Some(record) = self.nodes.get_mut(key) else {
                continue;
            };
            if record.dirty {
                continue;
            }
            record.dirty = true;
            stack.extend(record.children.iter().copied());
        }

        Ok(())
    }

    fn highest_dirty_ancestor(&self, id: NodeId) -> Option<NodeId> {
        let mut current = id;
        if !self.get_record(current)?.dirty {
            return None;
        }

        while let Some(parent) = self.get_record(current).and_then(|record| record.parent) {
            if !self.get_record(parent).is_some_and(|record| record.dirty) {
                break;
            }
            current = parent;
        }

        Some(current)
    }

    fn update_subtree(&mut self, root: NodeId) {
        let mut stack = Vec::from([root]);
        while let Some(id) = stack.pop() {
            let Some(key) = self.key(id) else {
                continue;
            };
            let parent_matrix = self
                .nodes
                .get(key)
                .and_then(|record| record.parent)
                .and_then(|parent| self.get_record(parent))
                .map_or(Mat4::IDENTITY, |record| record.world_matrix);

            {
                let Some(record) = self.nodes.get_mut(key) else {
                    continue;
                };
                if record.dirty {
                    record.world_matrix = parent_matrix * record.node.transform.to_mat4();
                    record.dirty = false;
                }
            }

            if let Some(record) = self.nodes.get(key) {
                for index in (0..record.children.len()).rev() {
                    stack.push(record.children[index]);
                }
            }
        }
    }

    fn is_descendant(&self, node: NodeId, ancestor: NodeId) -> bool {
        let mut current = Some(node);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self.parent(id);
        }
        false
    }
}

impl Default for SceneGraph {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn id_index(id: NodeId) -> Option<usize> {
    let raw = id.get();
    if raw == 0 || raw > usize::MAX as u64 {
        None
    } else {
        Some(raw as usize)
    }
}
