use alloc::vec::Vec;
use core::cmp::Ordering;

use scenix_core::NodeId;
use scenix_math::{Aabb, Ray3};

/// Scene-level BVH leaf entry.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BvhEntry {
    /// Scene node represented by this entry.
    pub node_id: NodeId,
    /// World-space node bounds.
    pub aabb: Aabb,
}

impl BvhEntry {
    /// Creates a BVH entry from a node ID and world-space bounds.
    #[inline]
    pub const fn new(node_id: NodeId, aabb: Aabb) -> Self {
        Self { node_id, aabb }
    }
}

/// A compact BVH node.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BvhNode {
    /// World-space bounds of this node.
    pub aabb: Aabb,
    /// Left child index for internal nodes.
    pub left: u32,
    /// Right child index for internal nodes.
    pub right: u32,
    /// First entry index for leaves.
    pub start: u32,
    /// Entry count for leaves. Internal nodes have `count == 0`.
    pub count: u32,
}

impl BvhNode {
    /// Returns whether this node is a leaf.
    #[inline]
    pub const fn is_leaf(self) -> bool {
        self.count > 0
    }
}

/// Surface-area-heuristic BVH over scene node AABBs.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bvh {
    nodes: Vec<BvhNode>,
    entries: Vec<BvhEntry>,
    leaf_size: usize,
}

impl Bvh {
    /// Builds a BVH from world-space scene entries.
    pub fn build(entries: &[BvhEntry]) -> Self {
        Self::build_with_leaf_size(entries, 4)
    }

    /// Builds a BVH with a custom maximum leaf size.
    pub fn build_with_leaf_size(entries: &[BvhEntry], leaf_size: usize) -> Self {
        let mut bvh = Self {
            nodes: Vec::new(),
            entries: entries.to_vec(),
            leaf_size: leaf_size.max(1),
        };
        if !bvh.entries.is_empty() {
            bvh.build_node(0, bvh.entries.len());
        }
        bvh
    }

    /// Returns all nodes potentially hit by `ray`.
    pub fn traverse(&self, ray: Ray3) -> Vec<NodeId> {
        let mut node_ids = Vec::new();
        for entry_index in self.traverse_entry_indices(ray) {
            node_ids.push(self.entries[entry_index].node_id);
        }
        node_ids
    }

    /// Returns whether the BVH has no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of entries.
    #[inline]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of internal and leaf nodes.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn traverse_entry_indices(&self, ray: Ray3) -> Vec<usize> {
        let mut hits = Vec::new();
        if self.nodes.is_empty() {
            return hits;
        }

        let mut stack = Vec::from([0usize]);
        while let Some(index) = stack.pop() {
            let node = self.nodes[index];
            if ray.intersect_aabb(node.aabb).is_none() {
                continue;
            }
            if node.is_leaf() {
                let start = node.start as usize;
                let end = start + node.count as usize;
                for entry_index in start..end {
                    if ray.intersect_aabb(self.entries[entry_index].aabb).is_some() {
                        hits.push(entry_index);
                    }
                }
            } else {
                let left = node.left as usize;
                let right = node.right as usize;
                let left_t = ray.intersect_aabb(self.nodes[left].aabb);
                let right_t = ray.intersect_aabb(self.nodes[right].aabb);
                match (left_t, right_t) {
                    (Some(a), Some(b)) if a <= b => {
                        stack.push(right);
                        stack.push(left);
                    }
                    (Some(_), Some(_)) => {
                        stack.push(left);
                        stack.push(right);
                    }
                    (Some(_), None) => stack.push(left),
                    (None, Some(_)) => stack.push(right),
                    (None, None) => {}
                }
            }
        }
        hits
    }

    fn build_node(&mut self, start: usize, end: usize) -> usize {
        let node_index = self.nodes.len();
        let aabb = bounds_for(&self.entries[start..end]);
        self.nodes.push(BvhNode {
            aabb,
            left: 0,
            right: 0,
            start: start as u32,
            count: (end - start) as u32,
        });

        let count = end - start;
        if count <= self.leaf_size {
            return node_index;
        }

        let Some(split) = self.find_sah_split(start, end) else {
            return node_index;
        };

        let left = self.build_node(start, split);
        let right = self.build_node(split, end);
        self.nodes[node_index] = BvhNode {
            aabb,
            left: left as u32,
            right: right as u32,
            start: 0,
            count: 0,
        };
        node_index
    }

    fn find_sah_split(&mut self, start: usize, end: usize) -> Option<usize> {
        let count = end - start;
        if count <= 1 {
            return None;
        }

        let centers = center_bounds(&self.entries[start..end]);
        let extent = centers.max - centers.min;
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };
        if extent[axis].abs() <= 1.0e-6 {
            return None;
        }

        self.entries[start..end].sort_by(|a, b| {
            let lhs = a.aabb.center()[axis];
            let rhs = b.aabb.center()[axis];
            lhs.total_cmp(&rhs).then_with(|| {
                let lhs_id = a.node_id.get();
                let rhs_id = b.node_id.get();
                if lhs_id < rhs_id {
                    Ordering::Less
                } else if lhs_id > rhs_id {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
        });

        let mut prefix = Vec::with_capacity(count);
        let mut running = self.entries[start].aabb;
        prefix.push(running);
        for entry in &self.entries[start + 1..end] {
            running = running.merge(entry.aabb);
            prefix.push(running);
        }

        let mut suffix = alloc::vec![Aabb::ZERO; count];
        running = self.entries[end - 1].aabb;
        suffix[count - 1] = running;
        for offset in (0..count - 1).rev() {
            running = running.merge(self.entries[start + offset].aabb);
            suffix[offset] = running;
        }

        let mut best_split = count / 2;
        let mut best_cost = f32::INFINITY;
        for split in 1..count {
            let left_count = split as f32;
            let right_count = (count - split) as f32;
            let cost = prefix[split - 1].surface_area() * left_count
                + suffix[split].surface_area() * right_count;
            if cost < best_cost {
                best_cost = cost;
                best_split = split;
            }
        }

        Some(start + best_split)
    }
}

fn bounds_for(entries: &[BvhEntry]) -> Aabb {
    let Some((first, rest)) = entries.split_first() else {
        return Aabb::ZERO;
    };
    let mut bounds = first.aabb;
    for entry in rest {
        bounds = bounds.merge(entry.aabb);
    }
    bounds
}

fn center_bounds(entries: &[BvhEntry]) -> Aabb {
    let Some((first, rest)) = entries.split_first() else {
        return Aabb::ZERO;
    };
    let mut bounds = Aabb::new(first.aabb.center(), first.aabb.center());
    for entry in rest {
        let center = entry.aabb.center();
        bounds = bounds.merge(Aabb::new(center, center));
    }
    bounds
}
