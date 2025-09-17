use std::collections::HashMap;

pub(crate) struct Node {
    id: usize,
    labels: Vec<String>,
    properties: HashMap<String, String>,
}

pub(crate) struct Edge {
    from: usize,
    to: usize,
    label: String,
}

pub(crate) struct Graph {
    nodes: HashMap<usize, Node>,
    edges: HashMap<usize, Vec<Edge>>,
    next_id: usize,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, labels: Vec<String>, properties: HashMap<String, String>) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let node = Node {
            id,
            labels,
            properties,
        };

        self.nodes.insert(id, node);

        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, label: String) -> Result<(), ()> {
        let (from_exists, to_exists) = (
            self.nodes.get(&from).is_some(),
            self.nodes.get(&to).is_some(),
        );

        if !(from_exists && to_exists) {
            return Err(());
        }

        let edge = Edge { from, to, label };

        let edges = self.edges.entry(from).or_insert(Vec::new());
        edges.push(edge);

        Ok(())
    }

    pub fn get_node(&self, id: usize) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_neighbors(&self, id: usize) -> Option<Vec<usize>> {
        Some(self.edges.get(&id)?.iter().map(|e| e.to).collect())
    }

    pub fn dfs(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        let mut visited = HashMap::new();
        let mut path = Vec::new();

        if !self.m_dfs(from, to, &mut visited, &mut path) {
            return None;
        }

        Some(path)
    }

    fn m_dfs(
        &self,
        current: usize,
        to: usize,
        visited: &mut HashMap<usize, bool>,
        path: &mut Vec<usize>,
    ) -> bool {
        if visited.contains_key(&current) {
            return false;
        }

        visited.insert(current, true);
        path.push(current);

        if current == to {
            return true;
        }

        if let Some(edges) = self.edges.get(&current) {
            for edge in edges {
                if self.m_dfs(edge.to, to, visited, path) {
                    return true;
                }
            }
        }

        path.pop();

        false
    }
}
