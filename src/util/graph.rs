use std::collections::{HashMap, VecDeque};

pub struct Node<T> {
    pub name: String,
    pub data: T,
}

pub struct Edge<U: Copy> {
    pub src: usize,
    pub dst: usize,

    pub info: U,
}

pub struct Graph<T, U: Copy> {
    nodes: Vec<Node<T>>,
    edges: Vec<Edge<U>>,

    node_refs: HashMap<String, usize>,
    src_edge_refs: HashMap<String, Vec<(usize, usize)>>, // Key is src node, values are dst nodes, .0 is node refs, .1 is edge refs
    dst_edge_refs: HashMap<String, Vec<(usize, usize)>>, // Key is dst_node, values are src nodes, .0 is node refs, .1 is edge refs
}

impl <T, U: Copy> Graph<T, U> {
    pub fn new() -> Graph<T, U> {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),

            node_refs: HashMap::new(),
            src_edge_refs: HashMap::new(),
            dst_edge_refs: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, name: &str, data: T) {
        self.nodes.push(Node { name: name.to_string(), data });
        self.node_refs.insert(name.to_string(), self.nodes.len() - 1);

        self.src_edge_refs.insert(name.to_string(), Vec::new());
        self.dst_edge_refs.insert(name.to_string(), Vec::new());
    }

    pub fn add_edge(&mut self, src: &str, dst: &str, info: U) {
        self.edges.push(Edge { 
            src: *self.node_refs.get(src).unwrap(),
            dst: *self.node_refs.get(dst).unwrap(),

            info,
        });

        let src_edge_ref = self.get_node_ref(dst);
        let dst_edge_ref = self.get_node_ref(src);

        self.src_edge_refs.get_mut(src).unwrap().push((src_edge_ref, self.edges.len() - 1));
        self.dst_edge_refs.get_mut(dst).unwrap().push((dst_edge_ref, self.edges.len() - 1));
    }

    pub fn get_node(&self, name: &str) -> &Node<T> {
        &self.nodes[*self.node_refs.get(name).unwrap()]
    }

    pub fn get_src_node(&self, edge: &Edge<U>) -> &Node<T> {
        &self.nodes[edge.src]
    }

    pub fn get_dst_node(&self, edge: &Edge<U>) -> &Node<T> {
        &self.nodes[edge.dst]
    }

    fn get_node_ref(&self, name: &str) -> usize {
        *self.node_refs.get(name).unwrap()
    }

    pub fn get_prev_nodes(&self, dst: &str) -> Vec<&Node<T>> {
        let prev_node_refs = self.dst_edge_refs.get(dst).unwrap();
        let mut prev_nodes = Vec::<&Node<T>>::with_capacity(prev_node_refs.len());        

        for node_ref in prev_node_refs {
            prev_nodes.push(&self.nodes[node_ref.0]);
        }

        prev_nodes
    }

    pub fn get_prev_edges(&self, dst: &str) -> Vec<&Edge<U>> {
        let prev_edge_refs = self.dst_edge_refs.get(dst).unwrap();
        let mut prev_edges = Vec::<&Edge<U>>::with_capacity(prev_edge_refs.len());        

        for edge_ref in prev_edge_refs {
            prev_edges.push(&self.edges[edge_ref.1]);
        }

        prev_edges
    }

    pub fn get_next_nodes(&self, src: &str) -> Vec<&Node<T>> {
        let next_node_refs = self.src_edge_refs.get(src).unwrap();
        let mut next_nodes = Vec::<&Node<T>>::with_capacity(next_node_refs.len());        

        for node_ref in next_node_refs {
            next_nodes.push(&self.nodes[node_ref.0]);
        }

        next_nodes
    }

    pub fn get_next_edges(&self, src: &str) -> Vec<&Edge<U>> {
        let next_edge_refs = self.src_edge_refs.get(src).unwrap();
        let mut next_edges = Vec::<&Edge<U>>::with_capacity(next_edge_refs.len());        

        for edge_ref in next_edge_refs {
            next_edges.push(&self.edges[edge_ref.1]);
        }

        next_edges
    }

    fn get_prev_node_refs(&self, dst: &str) -> Vec<usize> {
        self.dst_edge_refs.get(dst).unwrap().to_vec().iter().map(|e| {
            e.0
        }).collect()
    }

    fn get_next_node_refs(&self, src: &str) -> Vec<usize> {
        self.src_edge_refs.get(src).unwrap().to_vec().iter().map(|e| {
            e.0
        }).collect()
    }

    pub fn breadth_first_forwards(&self, root: &str) -> Vec<&Node<T>> {
        const MAX_ITERATIONS: u32 = 1000;

        let mut tree = vec![self.get_node(root)];
        let mut tree_refs = vec![self.get_node_ref(root)];

        let open_node_refs = self.get_next_node_refs(root);
        let mut open_node_refs_queue = VecDeque::from(open_node_refs);

        let mut iterations: u32 = 0;
        while !open_node_refs_queue.is_empty() && iterations < MAX_ITERATIONS {
            iterations += 1;

            let new_ref = open_node_refs_queue.pop_front().unwrap();
            let current_node = &self.nodes[new_ref];

            if !tree_refs.contains(&new_ref) {
                tree_refs.push(new_ref);
                tree.push(&self.nodes[new_ref]);
            }

            let next_open_node_refs = self.get_next_node_refs(&current_node.name);
            for node_ref in next_open_node_refs {
                if !open_node_refs_queue.contains(&node_ref) {
                    open_node_refs_queue.push_back(node_ref);
                }
            }
        }

        tree
    }

    pub fn breadth_first_backwards(&self, root: &str) -> Vec<&Node<T>> {
        const MAX_ITERATIONS: u32 = 1000;

        let mut tree = vec![self.get_node(root)];
        let mut tree_refs = vec![self.get_node_ref(root)];

        let open_node_refs = self.get_prev_node_refs(root);
        let mut open_node_refs_queue = VecDeque::from(open_node_refs);

        let mut iterations: u32 = 0;
        while !open_node_refs_queue.is_empty() && iterations < MAX_ITERATIONS {
            iterations += 1;

            let new_ref = open_node_refs_queue.pop_front().unwrap();
            let current_node = &self.nodes[new_ref];

            if !tree_refs.contains(&new_ref) {
                tree_refs.push(new_ref);
                tree.push(&self.nodes[new_ref]);
            }

            let prev_open_node_refs = self.get_prev_node_refs(&current_node.name);
            for node_ref in prev_open_node_refs {
                if !open_node_refs_queue.contains(&node_ref) {
                    open_node_refs_queue.push_back(node_ref);
                }
            }
        }

        tree
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }
}