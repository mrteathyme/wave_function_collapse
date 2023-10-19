#![feature(lazy_cell)]
use std::hash::Hash;
use std::collections::HashMap;
use std::fmt::Debug;

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::thread_rng;

use std::sync::{LazyLock, Mutex};
use std::thread;

static MAX_THREADS: LazyLock<Mutex<usize>> = LazyLock::new(Mutex::default);
static CHUNK_LIMIT: LazyLock<Mutex<usize>> = LazyLock::new(Mutex::default);


pub fn set_max_threads(thread_count: usize) {
    *MAX_THREADS.lock().unwrap() = thread_count;
}

pub fn set_chunk_limit(thread_count: usize) {
    *CHUNK_LIMIT.lock().unwrap() = thread_count;
}



#[derive(Clone, Debug)]
pub struct NodeSystem<NodeType>
    where NodeType: Node + Sized
{
    atlas: HashMap<NodeType::PositionType, NodeType>,
    queue: HashMap<NodeType::PositionType, f32>,
    domain_cache: HashMap<Box<str>, bool>
}

fn get_lowest_entropy<PositionType: Default + Clone>(entropy_list: Vec<(PositionType, f32)>) -> (PositionType, f32){
    let mut lowest_entropy = f32::MAX;
    let mut lowest_node = PositionType::default();
    for (position, entropy) in entropy_list {
        if entropy < lowest_entropy {
                    lowest_entropy = entropy.clone();
                    lowest_node = position.clone();
        }
    }
    (lowest_node,lowest_entropy)
}

impl<NodeType> Iterator for NodeSystem<NodeType>
    where NodeType: Node + Sized, NodeType::PositionType: 'static
{
    type Item = NodeType;

    fn next(&mut self) -> Option<Self::Item> {
        let mut lowest_node = NodeType::PositionType::default();
        if self.queue.len() > 1 {
            let chunk_size = self.queue.len() / *MAX_THREADS.lock().unwrap();
            if chunk_size > *CHUNK_LIMIT.lock().unwrap() {
                let vector = self.queue.clone().into_iter().collect::<Vec<_>>();
                let remainder = vector.len() - chunk_size * *MAX_THREADS.lock().unwrap();
                let mut chunks = vec![];
                let mut index: usize = 0;
                for i in 0..*MAX_THREADS.lock().unwrap() {
                    chunks.push(vector[index..index+chunk_size].to_vec().clone());
                    index+=chunk_size;
                }
                if remainder > 0 {
                    chunks.push(vector[index..index+remainder].to_vec().clone());
                }
                let mut threads = vec![];
                for chunk in chunks.into_iter() {
                    threads.push(thread::spawn(move || get_lowest_entropy::<NodeType::PositionType>(chunk)))
                }
                let mut lowest_entropy = f32::MAX;
                for thread in threads {
                   let (position, entropy) = thread.join().unwrap();
                   if entropy < lowest_entropy {
                        lowest_entropy = entropy;
                        lowest_node = position;
                   }
                }
            } else {


            let mut lowest_entropy = f32::MAX;
            for (position, entropy) in &self.queue {
                if entropy < &lowest_entropy {
                    lowest_entropy = entropy.clone();
                    lowest_node = position.clone();
                } 
            }}
        } else {
            for (atlas_position, atlas_node) in &self.atlas {
                if atlas_node.has_collapsed() == false {
                    lowest_node = atlas_position.clone();
                }
            }
        }
        if self.atlas.get(&lowest_node).unwrap().has_collapsed() == true {
            return None;
        }
        self.queue.remove(&lowest_node);
        let collapse_node = self.collapse_node(&lowest_node);
        self.atlas.insert(lowest_node, collapse_node.clone());
        Some(collapse_node)
    }
}

impl<NodeType> NodeSystem<NodeType>
    where NodeType: Node + Sized
{ 
    pub fn new() -> Self {
        Self {
            atlas: HashMap::new(),
            queue: HashMap::new(),
            domain_cache: HashMap::new(),
        }
    }
  
    pub fn get_cache(&self) -> HashMap<Box<str>, bool> {
        self.domain_cache.clone()
    }

    pub fn set_cache(&mut self, cache:HashMap<Box<str>, bool>) {
        self.domain_cache = cache;
    }


    pub fn reset_node(&mut self, position: &NodeType::PositionType) {
        self.atlas.insert(position.clone(), NodeType::new(position));
    }

    pub fn get_node(&self, position: &NodeType::PositionType) -> NodeType {
        self.atlas.get(&position).unwrap().clone()
    }

    pub fn get_atlas(&self) -> HashMap<NodeType::PositionType, NodeType> {
        self.atlas.clone()
    }

    pub fn add_node_to_atlas(&mut self, position: &NodeType::PositionType) {
        self.atlas.insert(position.clone(), NodeType::new(position));
    }
    
    pub fn add_node_to_queue(&mut self, position: &NodeType::PositionType) {
        match self.atlas.get(position) {
            Some(node) => {
                if node.has_collapsed() == false {
                    self.queue.insert(position.clone(), node.get_entropy());
                }
            }
            None => {}
        }
    }

    pub fn get_connecting_node_type_names(&self, nodes: &Vec<NodeType::NodeVariants>) -> Vec<Box<str>> {
        let mut names = vec![];
        for node in nodes {
            names.push(node.get_name());
        }
        names
    }

    pub fn get_connecting_node_types(&self, nodes: &Vec<NodeType::PositionType>) -> Vec<NodeType::NodeVariants> {
        let mut node_types = vec![];
        for position in nodes {
            match self.atlas.get(&position) {
                None => {continue},
                Some(node) => node_types.push(node.get_node_type())
            }
        }
        node_types
    }

    fn collapse_node(&mut self, position: &NodeType::PositionType) -> NodeType {
        let mut node = match self.atlas.get(&position) {
            None => NodeType::new(position),
            Some(node) => node.clone()
        };
        let mut choices = vec![];
        let mut weights = vec![];
        for (choice, weight) in node.get_domain_weights() {
            let mut can_spawn = true;
            let node_types = self.get_connecting_node_types(&node.get_connecting_nodes());
            let node_names = self.get_connecting_node_type_names(&self.get_connecting_node_types(&node.get_connecting_nodes()));
            let mut key = String::from(choice.get_name());
            for name in node_names {
                key.push_str(&name)
            }
            match self.domain_cache.get(&Box::<str>::from(key.clone())) {
                Some(result) => can_spawn = *result,
                None => {
                        for node in node_types.clone() {
                            if node.can_spawn_next_to(choice) == false { can_spawn = false; break;}
                        }
                        if can_spawn == true {
                            can_spawn = choice.can_spawn(node_types.clone());
                        }
                        self.domain_cache.insert(key.into(),can_spawn);
                }
            }
            if weight > 0.0 && can_spawn == true {
                choices.push(choice);
                weights.push(weight * choice.get_weighting_multiplier(&node_types));
            }
        }
        if weights.len() == 0 {
            node.set_node_type(NodeType::NodeVariants::get_invalid_type());
            node.set_collapsed(true);
            return node;
        }
        let dist = WeightedIndex::new(&weights).unwrap();
        let mut rng = thread_rng();
        node.set_node_type(choices[dist.sample(&mut rng)].clone());
        node.set_collapsed(true);
        self.atlas.insert(position.clone(), node.clone());
        node
    }
}


pub trait NodeVariant<const DOMAINSIZE: usize, Parent: NodeVariants> {
    const DOMAIN: [Parent; DOMAINSIZE];
    fn get_domain() -> Vec<Parent> {
        Self::DOMAIN.into()
    }
    fn can_spawn_next_to(node_type: Parent) -> bool {
       Self::DOMAIN.contains(&node_type) 
    }
}


pub trait NodeVariants
    where Self: Sized + PartialEq + Hash + Eq + Clone + Copy
{
    fn get_domain() -> Vec<Self>;
    fn get_domain_weights() -> HashMap<Self, f32> {
        let mut weights = HashMap::new();
        for variant in Self::get_domain() {
            weights.insert(variant, variant.get_weight());
        }
        weights
    }
    fn get_name(&self) -> Box<str>;
    fn get_weight(&self) -> f32;
    fn get_invalid_type() -> Self;
    fn can_spawn_next_to(&self, node_type: Self) -> bool;
    fn can_spawn(&self, node_types: Vec<Self>) -> bool;
    fn get_weighting_multiplier(&self, nodes: &Vec<Self>) -> f32;
}

pub trait Node: Clone
    where Self: Sized
{
    type PositionType: Hash + Eq + PartialEq + Clone + Debug + Default + Send + Sync + Sized;
    type NodeVariants: NodeVariants + Clone; 
    fn new(position: &Self::PositionType) -> Self;
    fn get_atlas_position(&self) -> Self::PositionType;
    fn get_connecting_nodes(&self) -> Vec<Self::PositionType>;
    fn get_node_type(&self) -> Self::NodeVariants;
    fn set_node_type(&mut self, node_type: Self::NodeVariants);
    fn get_default_type() -> Self::NodeVariants;
    fn get_domain_weights(&self) -> HashMap<Self::NodeVariants, f32>; 
    fn get_entropy(&self) -> f32 {
        let mut weighting_sum = 0.0;
        let mut log_weights_sum = 0.0;
        for (_, weight) in self.get_domain_weights() {
            if weight > 0.0 {
                weighting_sum += weight;
                log_weights_sum += weight * weight.log2();
            }
        }
        if weighting_sum <= 0.0 {return f32::MAX};
        weighting_sum.log2() - (log_weights_sum / weighting_sum)
    }
    fn has_collapsed(&self) -> bool;
    fn set_collapsed(&mut self, state: bool);
}
