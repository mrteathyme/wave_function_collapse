use wave_function_collapse::*;
use std::collections::HashMap;
use std::ops::Add;

use std::time::Instant;

use bmp_rust::bmp::BMP;

use std::fs;
use toml::Table;
use serde::{Serialize,Deserialize};


const DIRECTIONS: [Position; 8] = [
    Position{x:-1,y:1,z:0},
    Position{x:0,y:1,z:0},
    Position{x:1,y:1,z:0},
    Position{x:-1,y:0,z:0},
    Position{x:1,y: 0,z:0},
    Position{x:-1,y:-1,z:0},
    Position{x:0,y:-1,z:0},
    Position{x:1,y:-1,z:0},
];

static RESOLUTION: (i32, i32) = (3440, 1440);



fn collapse_tiles(node_system: &mut NodeSystem<TileNode>, pass_counter: i32) -> i32 {
    let mut invalid_tiles = 0;
    println!("Beginning Pass {pass_counter}");
    let timestamp = Instant::now();
    while let Some(collapsed_node) = node_system.next() {
        if collapsed_node.get_node_type() == TileType::Invalid {invalid_tiles+=1}
        for position in collapsed_node.get_connecting_nodes() {
            node_system.add_node_to_queue(&position.clone());
        }
    }
    let elapsed = timestamp.elapsed().as_micros();
    let seconds = elapsed as f32 / 1000000.0;
    let node_count = (RESOLUTION.0/10)*(RESOLUTION.1/10);
    let nodes_per_second = node_count as f32 / ((elapsed as f32) /1000000.0);
    println!("Finished collapsing {node_count} nodes in {seconds} seconds: {nodes_per_second} nodes per second, Spawned {invalid_tiles} invalid nodes");
    println!("Scanning for additional invalid nodes");
    for (position, node) in node_system.get_atlas() {
        if node.get_node_type() == TileType::Invalid {
                for connecting_node in node.get_connecting_nodes() {
                    node_system.reset_node(&connecting_node);
                    invalid_tiles += 1;
                }
            node_system.reset_node(&position);
            invalid_tiles +=1;
            continue
        }
        if node.get_node_type().can_spawn(node_system.get_connecting_node_types(&node.get_connecting_nodes())) == false {
            invalid_tiles += 1;
            node_system.reset_node(&position);
        }
    }
    println!("Reset {invalid_tiles} nodes");
    invalid_tiles
}

fn main() {
    set_max_threads(24);
    set_chunk_limit(50000);
    let mut node_system: NodeSystem<TileNode> = NodeSystem::new();
    let cache_toml: HashMap<Box<str>, bool> = toml::from_str(&fs::read_to_string("cache.toml").unwrap()).unwrap(); //(fs::read("cache.toml").unwrap()).unwrap();
    node_system.set_cache(cache_toml);
    for x in 0..(RESOLUTION.0/10) {
        for y in 0..(RESOLUTION.1/10) {
            node_system.add_node_to_atlas(&Position {x,y,z:0});
        }
    }
    
    let mut invalid_tiles = 1;
    let mut pass_counter = 1;
    while invalid_tiles > 0 {
        invalid_tiles = collapse_tiles(&mut node_system, pass_counter);
        pass_counter += 1;
    }

    let mut img = BMP::new(RESOLUTION.1, RESOLUTION.0 as u32, None);
    for x in 0..(RESOLUTION.0/10) {
        for y in 0..(RESOLUTION.1/10) {
            let color: [u8; 4] = match node_system.get_node(&Position{x, y, z:0}).get_node_type() {
                    TileType::Grass => [0, 255, 0, 255],
                    TileType::Water => [0, 179, 255, 255],
                    TileType::DeepWater => [0,0,255,255],
                    //TileType::BigTree => [21,76,0,255],
                    TileType::Sand => [255, 255, 0, 255],
                    TileType::Tree => [92, 108, 0, 255],
                    //TileType::House => [245, 40, 145, 255],
                    _ => [0, 0, 0, 255],
                };
                let _ = img.draw_rectangle(
                    Some(color),
                    Some(color),
                    [(x * 10) as u16, (y * 10) as u16],
                    [((x + 1) * 10 - 1) as u16, ((y + 1) * 10 - 1) as u16],
                );
            }
        }
    let _ = img.save_to_new("img.bmp");
    let cache_toml = toml::to_string(&node_system.get_cache()).unwrap(); 
    fs::write("cache.toml", cache_toml).unwrap();
}


impl Node for TileNode {
    type PositionType = Position;
    type NodeVariants = TileType;

    fn new(position: &Position) -> Self {
        let mut connecting_nodes = vec![];
        for direction in DIRECTIONS {
            connecting_nodes.push(*position + direction);
        }
        Self {
            tile_type: TileType::None,
            position: position.clone(),
            connecting_nodes,
            collapsed:false
        }
    }
    fn get_atlas_position(&self) -> Self::PositionType {
        self.position
    }
    fn get_node_type(&self) -> Self::NodeVariants {
        self.tile_type
    }
    fn set_node_type(&mut self, node_type: Self::NodeVariants) {
        self.tile_type = node_type
    }
    fn get_default_type() -> Self::NodeVariants {
        Self::NodeVariants::None
    }
    fn get_connecting_nodes(&self) -> Vec<Self::PositionType> {
        self.connecting_nodes.clone()
    }
    fn get_domain_weights(&self) -> HashMap<Self::NodeVariants, f32> {
        Self::NodeVariants::get_domain_weights()
    }
    fn has_collapsed(&self) -> bool {
        self.collapsed
    }
    fn set_collapsed(&mut self, state: bool) {
        self.collapsed = state;
    }
}



#[derive(Clone, Debug)]
struct TileNode {
    tile_type: TileType,
    position: Position,
    connecting_nodes: Vec<Position>,
    collapsed: bool
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
struct Position { x: i32, y: i32, z: i32 }

impl Add for Position {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x, 
            y: self.y + other.y, 
            z: self.z + other.z,
        }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
enum TileType {
    None,
    Invalid,
    Sand,
    Grass,
    Water,
    Tree,
    DeepWater,
    //DeeperWater,
    //BigTree,
}

impl NodeVariants for TileType { 
    fn get_domain() -> Vec<Self> {
        vec![Self::Sand, Self::Grass, Self::Water, Self::DeepWater, Self::Tree]
    }
   
    fn get_name(&self) -> Box<str> {
        match self {
        Self::Sand => "Sand".into(),
        Self::Water => "Water".into(),
        Self::Grass => "Grass".into(),
        Self::DeepWater => "DeepWater".into(),
        Self::Tree => "Tree".into(),
        Self::Invalid => "Invalid".into(),
        Self::None => "None".into(),
    }}

    fn get_weight(&self) -> f32 {
        match self {
            Self::Sand => 0.1,
            Self::Water => 1.0,
            Self::Grass => 1.0,
            Self::DeepWater => 0.9,
            Self::Tree => 0.9,
            _ => 1.0
        }
    }
    
    fn get_weighting_multiplier(&self, nodes: &Vec<Self>) -> f32 {
        match self {
            //Self::Tree => {
            //    let mut multiplier = 1.0;
            //    for node in nodes {
            //        if node == &Self::Grass {multiplier += 1.0};
            //        if node == &Self::Tree {multiplier += 1.0};
            //        if node == &Self::BigTree {multiplier += 1.0};
            //    }
            //    multiplier
            //},
            //Self::DeepWater => {
            //    let mut multiplier = 1.0;
            //    for node in nodes {
            //        if node == &Self::Water {multiplier += 1.0};
            //        if node == &Self::DeepWater {multiplier += 1.0};
            //        if node == &Self::DeeperWater {multiplier += 1.0};
            //    }
            //    multiplier 
            //}
            _ => 1.0
        }
    }

    fn get_invalid_type() -> Self {
       Self::Invalid 
    }
    
    fn can_spawn(&self, node_types: Vec<Self>) -> bool {
        match self {    
            Self::Sand => {
                let mut grass_count = 0;
                let mut water_count = 0;
                for node in node_types {
                    if node == Self::Grass {grass_count += 1};
                    if node == Self::Water {water_count += 1};
                }
                if grass_count > 5 || water_count > 5 { false } else { true }
            },
            Self::Water => {
                let mut deep_count = 0;
                for node in node_types {
                    if node == Self::DeepWater {deep_count += 1};
                }
                if deep_count > 5 { false } else { true }
            },
            Self::Grass => {
                let mut tree_count = 0;
                for node in node_types {
                    if node == Self::Tree {tree_count += 1};
                }
                if tree_count > 5 { false } else { true }
            }
            _ => true
        }
    }

    fn can_spawn_next_to(&self, node_type: Self) -> bool {
        match self {
            Self::Sand => [Self::Sand, Self::Water, Self::Grass,Self::None, Self::Invalid].contains(&node_type),
            Self::Grass => [Self::Tree, Self::Grass, Self::Sand,Self::None, Self::Invalid].contains(&node_type),
            Self::Water => [Self::DeepWater, Self::Water, Self::Sand,Self::None, Self::Invalid].contains(&node_type),
            Self::DeepWater => [Self::Water, Self::DeepWater, Self::None, Self::Invalid].contains(&node_type),
            Self::Tree => [Self::Tree, Self::Grass, Self::None, Self::Invalid].contains(&node_type),
            //Self::BigTree => [Self::BigTree, Self::Tree, Self::None, Self::Invalid].contains(&node_type),
            //Self::DeeperWater => [Self::DeepWater, Self::Water, Self::None, Self::Invalid].contains(&node_type),
            _ => true
        }
    }
}
