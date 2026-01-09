use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Directed Acyclic Graph for managing dependencies between formulas
#[derive(Debug, Clone)]
pub struct DAGraph<K, V>
where
    K: Eq + Hash + Clone,
{
    data: HashMap<K, V>,
    incoming_edges: HashMap<K, HashSet<K>>,
    outgoing_edges: HashMap<K, HashSet<K>>,
}

impl<K, V> DAGraph<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            incoming_edges: HashMap::new(),
            outgoing_edges: HashMap::new(),
        }
    }

    /// Add a node with its data and outgoing edges (dependencies)
    pub fn add_node(&mut self, key: K, data: V, outgoing: Vec<K>) -> Result<(), String> {
        if self.outgoing_edges.contains_key(&key) {
            return Err("Node with the provided key already exists".to_string());
        }

        self.data.insert(key.clone(), data);
        self.add_edges(key, outgoing);
        Ok(())
    }

    /// Get data for a specific key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    /// Check if a key exists in the graph
    pub fn contains(&self, key: &K) -> bool {
        self.outgoing_edges.contains_key(key)
    }

    /// Add edges from a key to its dependencies
    fn add_edges(&mut self, key: K, outgoing: Vec<K>) {
        let outgoing_set: HashSet<K> = outgoing.into_iter().collect();
        
        for dest in &outgoing_set {
            self.incoming_edges
                .entry(dest.clone())
                .or_insert_with(HashSet::new)
                .insert(key.clone());
        }
        
        self.outgoing_edges.insert(key, outgoing_set);
    }

    /// Perform topological sort, returning layers of nodes that can be executed in parallel
    /// Returns (layers, detached) where detached nodes have dependencies that don't exist
    pub fn topological_sort(&self) -> (Vec<Vec<K>>, Vec<K>) {
        let mut layers: Vec<Vec<K>> = vec![vec![]];
        let mut detached: Vec<K> = vec![];

        // Find nodes with no outgoing edges (first layer) and detached nodes
        for (key, destinations) in &self.outgoing_edges {
            if destinations.is_empty() {
                layers[0].push(key.clone());
            } else if destinations.iter().any(|dest| !self.outgoing_edges.contains_key(dest)) {
                detached.push(key.clone());
            }
        }

        let mut satisfied_keys: HashSet<K> = layers[0].iter().cloned().collect();
        let mut unsatisfied_keys: HashSet<K> = HashSet::new();

        while !layers.last().unwrap().is_empty() {
            let mut candidates: HashSet<K> = HashSet::new();
            
            // Get all nodes that point to nodes in the previous layer
            for prev in layers.last().unwrap() {
                if let Some(incoming) = self.incoming_edges.get(prev) {
                    for key in incoming {
                        if self.outgoing_edges.contains_key(key) {
                            candidates.insert(key.clone());
                        }
                    }
                }
            }
            
            // Add previously unsatisfied keys
            candidates.extend(unsatisfied_keys.drain());

            let mut current_level: Vec<K> = vec![];
            
            for candidate in candidates {
                // Check if all dependencies are satisfied
                let all_satisfied = self.outgoing_edges[&candidate]
                    .iter()
                    .all(|dep| satisfied_keys.contains(dep));

                if all_satisfied {
                    current_level.push(candidate.clone());
                    satisfied_keys.insert(candidate);
                } else {
                    unsatisfied_keys.insert(candidate);
                }
            }

            layers.push(current_level);
        }

        // Remove the last empty layer
        layers.pop();
        
        // Add remaining unsatisfied keys to detached
        detached.extend(unsatisfied_keys);

        (layers, detached)
    }
}

impl<K, V> Default for DAGraph<K, V>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph: DAGraph<String, i32> = DAGraph::new();
        let (layers, detached) = graph.topological_sort();
        assert_eq!(layers.len(), 0);
        assert_eq!(detached.len(), 0);
    }

    #[test]
    fn test_simple_dependency() {
        let mut graph = DAGraph::new();
        graph.add_node("a".to_string(), 1, vec![]).unwrap();
        graph.add_node("b".to_string(), 2, vec!["a".to_string()]).unwrap();
        
        let (layers, detached) = graph.topological_sort();
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], vec!["a".to_string()]);
        assert_eq!(layers[1], vec!["b".to_string()]);
        assert_eq!(detached.len(), 0);
    }

    #[test]
    fn test_parallel_execution() {
        let mut graph = DAGraph::new();
        graph.add_node("a".to_string(), 1, vec![]).unwrap();
        graph.add_node("b".to_string(), 2, vec![]).unwrap();
        graph.add_node("c".to_string(), 3, vec!["a".to_string(), "b".to_string()]).unwrap();
        
        let (layers, detached) = graph.topological_sort();
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].len(), 2);
        assert!(layers[0].contains(&"a".to_string()));
        assert!(layers[0].contains(&"b".to_string()));
        assert_eq!(layers[1], vec!["c".to_string()]);
        assert_eq!(detached.len(), 0);
    }

    #[test]
    fn test_detached_nodes() {
        let mut graph = DAGraph::new();
        graph.add_node("a".to_string(), 1, vec!["missing".to_string()]).unwrap();
        
        let (layers, detached) = graph.topological_sort();
        assert_eq!(detached.len(), 1);
        assert_eq!(detached[0], "a".to_string());
    }
}
