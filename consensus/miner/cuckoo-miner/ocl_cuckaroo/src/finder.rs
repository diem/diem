use hashbrown::HashMap;

#[derive(Clone)]
pub struct Solution {
    pub nodes: Vec<u32>,
}

pub struct Graph {
    adj_index: HashMap<u32, usize>,
    adj_store: Vec<AdjNode>,
}

struct Search {
    path: Vec<u32>,
    solutions: Vec<Solution>,

    state: HashMap<u32, NodeState>,
    node_visited: usize,
}

#[derive(Clone, Copy)]
enum NodeState {
    NotVisited,
    Visited,
}

impl Search {
    fn new(node_count: usize) -> Search {
        Search {
            path: Vec::with_capacity(node_count),
            solutions: vec![],
            state: HashMap::with_capacity_and_hasher(node_count, Default::default()),
            node_visited: 0,
        }
    }

    #[inline]
    fn visit(&mut self, node: u32) {
        self.state.insert(node, NodeState::Visited);
        self.path.push(node);
        self.node_visited += 1;
    }

    #[inline]
    fn leave(&mut self, node: u32) {
        self.path.pop();
        self.state.insert(node, NodeState::NotVisited);
    }

    #[inline]
    fn state(&self, node: u32) -> NodeState {
        match self.state.get(&node) {
            None => NodeState::NotVisited,
            Some(state) => *state,
        }
    }

    #[inline]
    fn is_visited(&self, node: u32) -> bool {
        match self.state(node) {
            NodeState::NotVisited => false,
            _ => true,
        }
    }
}

struct AdjNode {
    value: u32,
    next: Option<usize>,
}

impl AdjNode {
    #[inline]
    fn first(value: u32) -> AdjNode {
        AdjNode { value, next: None }
    }

    #[inline]
    fn next(value: u32, next: usize) -> AdjNode {
        AdjNode {
            value,
            next: Some(next),
        }
    }
}

struct AdjList<'a> {
    current: Option<&'a AdjNode>,
    adj_store: &'a Vec<AdjNode>,
}

impl<'a> AdjList<'a> {
    #[inline]
    pub fn new(current: Option<&'a AdjNode>, adj_store: &'a Vec<AdjNode>) -> AdjList<'a> {
        AdjList { current, adj_store }
    }
}

impl<'a> Iterator for AdjList<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                let val = node.value;
                match node.next {
                    None => self.current = None,
                    Some(next_index) => self.current = Some(&self.adj_store[next_index]),
                }
                Some(val)
            }
        }
    }
}

impl Graph {
    pub fn search(nodes: &[u32]) -> Result<Vec<Solution>, String> {
        let edge_count = nodes.len() / 2;
        let mut g = Graph {
            adj_index: HashMap::with_capacity_and_hasher(nodes.len(), Default::default()),
            adj_store: Vec::with_capacity(nodes.len()),
        };
        let mut search = Search::new(nodes.len());
        const STEP: usize = 2;
        for i in 0..edge_count {
            let n1 = nodes[i * STEP];
            let n2 = nodes[i * STEP + 1];
            g.walk_graph(n1, n2, &mut search)?;
            g.add_edge(n1, n2);
        }

        Ok(search.solutions.clone())
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.adj_index.len()
    }

    #[inline]
    pub fn edge_count(&self) -> usize {
        self.adj_store.len() / 2
    }

    #[inline]
    fn add_edge(&mut self, node1: u32, node2: u32) {
        self.add_half_edge(node1, node2);
        self.add_half_edge(node2, node1);
    }

    fn add_half_edge(&mut self, from: u32, to: u32) {
        if let Some(index) = self.adj_index.get(&from) {
            self.adj_store.push(AdjNode::next(to, *index));
        } else {
            self.adj_store.push(AdjNode::first(to));
        }
        self.adj_index.insert(from, self.adj_store.len() - 1);
    }

    fn neighbors(&self, node: u32) -> Option<impl Iterator<Item = u32> + '_> {
        let node = match self.adj_index.get(&node) {
            Some(index) => Some(&self.adj_store[*index]),
            None => return None,
        };
        Some(AdjList::new(node, &self.adj_store))
    }

    fn walk_graph(&self, current: u32, target: u32, search: &mut Search) -> Result<(), String> {
        if search.path.len() > 41 {
            return Ok(());
        }

        let neighbors = match self.neighbors(current) {
            None => return Ok(()),
            Some(it) => it,
        };
        search.visit(current);
        for ns in neighbors {
            if ns == target && search.path.len() == 41 {
                search.path.push(ns);
                search.solutions.push(Solution {
                    nodes: search.path.clone(),
                });
                search.leave(current);
                return Ok(());
            }
            if !search.is_visited(ns) {
                self.walk_graph(ns, target, search)?;
            }
        }
        search.leave(current);
        Ok(())
    }
}
