use alloc::vec::Vec;

use scenix_core::NodeId;

use crate::SceneGraph;

/// Depth-first scene graph node ID iterator.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DepthFirstIter<'a> {
    graph: &'a SceneGraph,
    stack: Vec<NodeId>,
}

impl<'a> DepthFirstIter<'a> {
    pub(crate) fn new(graph: &'a SceneGraph) -> Self {
        let mut stack = graph.roots().to_vec();
        stack.reverse();
        Self { graph, stack }
    }
}

impl Iterator for DepthFirstIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.stack.pop()?;
        if let Some(children) = self.graph.children(id) {
            self.stack.extend(children.iter().rev().copied());
        }
        Some(id)
    }
}

/// Breadth-first scene graph node ID iterator.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct BreadthFirstIter<'a> {
    graph: &'a SceneGraph,
    queue: Vec<NodeId>,
    index: usize,
}

impl<'a> BreadthFirstIter<'a> {
    pub(crate) fn new(graph: &'a SceneGraph) -> Self {
        Self {
            graph,
            queue: graph.roots().to_vec(),
            index: 0,
        }
    }
}

impl Iterator for BreadthFirstIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = *self.queue.get(self.index)?;
        self.index += 1;
        if let Some(children) = self.graph.children(id) {
            self.queue.extend(children.iter().copied());
        }
        Some(id)
    }
}
