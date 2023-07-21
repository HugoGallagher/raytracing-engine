use std::collections::HashMap;

pub struct Node<T> {
    pub data: T,
}

#[derive(Copy, Clone)]
pub struct Edge<U: Copy> {
    pub info: U,
}

pub struct Graph<T, U: Copy> {
    pub nodes: HashMap<String, Node<T>>,
    src_edges: HashMap<String, Vec<(String, Option<Edge<U>>)>>, // Key is src node, values are dst nodes
    dst_edges: HashMap<String, Vec<(String, Option<Edge<U>>)>>, // Key is dst_node, values are src nodes and edge info
}

impl <T, U: Copy> Graph<T, U> {
    pub fn new() -> Graph<T, U> {
        Graph {
            nodes: HashMap::new(),
            src_edges: HashMap::new(),
            dst_edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, key: &str, data: T) {
        self.nodes.insert(key.to_string(), Node { data });

        self.src_edges.insert(key.to_string(), Vec::new());
        self.dst_edges.insert(key.to_string(), Vec::new());
    }

    pub fn add_edge(&mut self, src: &str, dst: &str, info: Option<Edge<U>>) {
        self.src_edges.get_mut(src).unwrap().push((dst.to_string(), info));
        self.dst_edges.get_mut(dst).unwrap().push((src.to_string(), info));

        //match self.src_edges.get(src) {
        //    Some(v) => { if !v.contains(&dst) { v.push(dst); } }
        //    None => { self.src_edges.insert(src, Vec::new()); }
        //}

        //match self.dst_edges.get(dst) {
        //    Some(v) => { if !v.contains(&src) { v.push(src); } }
        //    None => { self.dst_edges.insert(dst, Vec::new()); }
        //}
    }

    pub fn get_node(&self, name: &str) -> &Node<T> {
        self.nodes.get(name).unwrap()
    }

    pub fn get_prev(&self, dst: &str) -> &Vec<(String, Option<Edge<U>>)> {
        self.dst_edges.get(dst).unwrap()
    }

    pub fn get_next(&self, src: &str) -> &Vec<(String, Option<Edge<U>>)> {
        self.src_edges.get(src).unwrap()
    }
}