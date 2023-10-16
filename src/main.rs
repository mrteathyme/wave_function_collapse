use std::ops::Add;

use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

use rand::prelude::*;
use rand::distributions::WeightedIndex;


use std::fs::File;
use std::io::prelude::*;

use bmp_rust::bmp::BMP;

static DIRECTIONS: [Position; 8] = [
                                    Position(-1,1,0), Position(0,1,0), Position(1,1,0),
                                    Position(-1,0,0)                 , Position(1,0,0),
                                    Position(-1,-1,0),Position(0,-1,0),Position(1,-1,0)
                                   ];

fn main() {
    let stack = vec![];
    let mut tile_atlas: HashMap<Position, Tile> = HashMap::new();
    for x in 0..128 { 
        for y in 0..72 {
            let mut tile = Tile::new(Position(x,y,0));
            tile.collapse(&tile_atlas, &stack);
            tile_atlas.insert(Position(x,y,0), tile);
        }
    }

    let mut file = File::create("map.txt").unwrap();
    let mut string = String::new();
    let mut img = BMP::new(720, 1280, None);
    for x in 0..128 {
        for y in 0..72 {
            string.push(match tile_atlas.get(&Position(x, y, 0)).unwrap().tile_type {
                TileType::Grass => 'g',
                TileType::Sand => 's',
                TileType::Tree => 't',
                TileType::Water => 'w',
                TileType::House => 'h',
                TileType::None => ' ',
                TileType::Invalid => '?',
           });
            let color: [u8; 4] = match tile_atlas.get(&Position(x,y,0)).unwrap().tile_type {
                TileType::Grass => [0,255,0,255],
                TileType::Water => [0,0,255,255],
                TileType::Sand => [255,255,0,255],
                TileType::Tree => [92,108,0,255],
                TileType::House => [245,40,145,255],
                _ => [0,0,0,255]
            };
            img.draw_rectangle(Some(color), Some(color), [(x*10) as u16, (y*10) as u16], [((x+1)*10-1) as u16, ((y+1)*10-1) as u16]);
        }
        string.push('\n');
    }
    file.write_all(string.as_bytes()).unwrap();
    let _ = img.save_to_new("img.bmp");
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct Position(i32,i32,i32); 

impl Add for Position {
    type Output = Self;
    fn add(self, other:Self) -> Self {
        Self {
            0: self.0 + other.0,
            1: self.1 + other.1,
            2: self.2 + other.2
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    tile_type: TileType,
    position: Position,
    connecting_tiles: [Option<TileType>; 8]
}

impl Tile {
    fn new(position: Position) -> Tile {
        Tile {
            tile_type: TileType::None,
            position,
            connecting_tiles: [None;8]
        }
    }
    
    fn collapse(&mut self, tile_atlas: &HashMap<Position, Tile>, stack: &[Tile]) {
        self.update_connecting_tiles(tile_atlas);
        let mut choices = vec![];
        let mut weights = vec![];
        for (key,value) in self.get_domain_weights() {
           if value > 0.0 {
            choices.push(key);
            weights.push(value);
           }
        }
        if weights.len() == 0 {
            self.tile_type = TileType::Invalid;
            return
        }
        let dist = WeightedIndex::new(&weights).unwrap();
        let mut rng = thread_rng();
        self.tile_type = choices[dist.sample(&mut rng)];
    }


    
    fn update_connecting_tiles(&mut self, tile_atlas: &HashMap<Position, Tile>) {
        for (i, direction) in DIRECTIONS.into_iter().enumerate() {
            self.connecting_tiles[i] = match tile_atlas.get(&(direction+self.position)) {
                Some(tile) => Some(tile.tile_type),
                None => None
            };
        }
    }

    fn get_domain_weights(&self) -> HashMap<TileType, f32> {
        let mut weights = TileType::init_domain_weights();
        let real_tiles: Vec<TileType> = self.connecting_tiles.into_iter().filter(|&value| value != None).map(|value| value.unwrap()).collect();
        for (tile, _) in weights.clone() {
            if tile.can_spawn(&real_tiles) == false {
                weights.insert(tile, 0.0);
            }
        }
        weights
    }

    fn get_entropy(&self) -> f32 {
        let mut weighting_sum: f32 = 0.0;
        let mut log_weights_sum: f32 = 0.0;
        for (tile, weight) in self.get_domain_weights() {
            if weight > 0.0 {
                println!("{2} added: {0} to weights sum, {2} added {1} to log_weights", weight, weight*weight.log2(),format!("{:#?}", tile));
                weighting_sum += weight;
                log_weights_sum += weight * weight.log2(); 
            }
        }
        println!("Weighting sum is: {weighting_sum}  log_weights_sum is: {log_weights_sum}, log of weight sum is: {}", weighting_sum.log2());
        weighting_sum.log2() - (log_weights_sum / weighting_sum)
    }
}

struct Domain {
    domain_types: Vec<TileType>
}

impl Domain {

}

fn collapse_tile(tile_atlas: &HashMap<(i32,i32), TileType>, x_pos: i32, y_pos: i32) -> TileType {
    let mut domain = TileType::init_domain();
    let mut domain_keys = vec![];
    for (key,_) in domain.clone() {
        domain_keys.push(key);
    }

    let connecting_tiles = [
                (x_pos-1,y_pos-1),
                (x_pos-1,y_pos),
                 (x_pos-1,y_pos+1),
                (x_pos,y_pos-1),
                (x_pos,y_pos+1),
                (x_pos+1,y_pos-1),
                (x_pos+1,y_pos),
                (x_pos+1,y_pos+1)

    ];
    /*
    for tile in connecting_tiles {
        if let Some(tile_type) = tile_atlas.get(&tile) {
            for domain_key in domain_keys.clone() {
                if domain.get(&domain_key) == Some(&true) && tile_type.get_domain().contains(&domain_key) == false {
                    domain.insert(domain_key, false);
                }
            }
        }
    }
    */
   
    let mut connected_tiles = vec![];
    for tile in connecting_tiles {
        if let Some(tile_type) = tile_atlas.get(&tile) {
            connected_tiles.push(tile_type.clone());
        }
    }
    for domain_key in domain_keys.clone() {
        domain.insert(domain_key.clone(), domain_key.can_spawn(&connected_tiles));
    }

    let mut rng = thread_rng();
    let printable_domain = domain.clone();
    let mut filtered_domain = domain.into_iter().filter(|(key,value)| (value == &true)).map(|(tile_type, _)| tile_type).collect::<Vec<TileType>>(); 
    match filtered_domain.choose(&mut rng) {
        Some(tile) => {tile.clone()},
        None => {println!("{:#?}", printable_domain); println!("{:#?}", domain_keys); TileType::Invalid }
    }
}
#[derive(Clone, Debug, Hash, Eq, PartialEq, Copy)]
enum TileType {
    Grass,
    Water,
    Sand,
    Tree,
    House,
    None,
    Invalid,
}

impl TileType {
    const DEFAULT_DOMAINS: [TileType; 5] = [Self::Grass, Self::Water, Self::Sand, Self::Tree, Self::House];
    fn get_domain(&self) -> Vec<Self> {
        match self {
            Self::None => Self::DEFAULT_DOMAINS.into(),
            Self::Grass => [Self::Grass, Self::Tree, Self::Sand].into(),
            Self::Water => [Self::Water, Self::Sand].into(),
            Self::Tree => [Self::Tree, Self::Grass].into(),
            Self::Sand => [Self::Sand, Self::Water, Self::Grass].into(),
            Self::House => [Self::Grass].into(),
            _ => [Self::Invalid].into()
        }
    }

    fn can_spawn(&self, connecting_tiles: &[Self]) -> bool {
        match self {
            Self::House => {
                if thread_rng().gen_range(0..=100) <= 90 {
                    return false
                }
                for tile in connecting_tiles {
                    if [Self::Grass, Self::None, Self::Invalid].contains(tile) == false {
                        return false;
                    }
                }
                true
            },
            Self::Grass => {
                let mut water_weight = 8; //faking weighted spawning, will implement properly
                                          //later, this method leads too invalid states
                let mut tree_weight = 6;
                for tile in connecting_tiles {
                    if [Self::Grass, Self::House, Self::Sand, Self::Tree, Self::Water, Self::None, Self::Invalid].contains(tile) == false {
                        return false;
                    }
                    if tile == &Self::Water {
                        water_weight *= 2;
                    }
                    if tile == &Self::Tree {
                        tree_weight *= 2;
                    }
                }
                if connecting_tiles.contains(&Self::Tree) {
                    if thread_rng().gen_range(0..100) <= tree_weight {
                        return false;
                    }
                }
                if connecting_tiles.contains(&Self::Water) {
                if thread_rng().gen_range(0..100) <= water_weight {
                    return false;
                }
                }
                true
            }
            Self::Sand => {
                let mut water_weight = 7;
                for tile in connecting_tiles {
                   if [Self::Grass, Self::Sand, Self::Water, Self::Invalid, Self::None].contains(tile) == false {
                       return false;
                   }
                   if tile == &Self::Water {
                    water_weight *= 2;
                   }
                }
                if connecting_tiles.contains(&Self::Water) || connecting_tiles.contains(&Self::Sand) || connecting_tiles.contains(&Self::None) || connecting_tiles.contains(&Self::Invalid) {
                    if connecting_tiles.contains(&Self::Water) == false {
                        if thread_rng().gen_range(0..=100) <= 80 {
                            return false;
                        }
                    } else {
                        if thread_rng().gen_range(0..=100) <= water_weight {
                            return false;
                        }
                    }
                    return true;
                } 
                false
            },
            Self::Water => {
                for tile in connecting_tiles {
                    if [Self::Grass, Self::Sand, Self::Water, Self::None, Self::Invalid].contains(tile) == false {
                        return false;
                    }
                }
                if connecting_tiles.contains(&Self::Sand) == false && connecting_tiles.contains(&Self::Water) == false {
                    if thread_rng().gen_range(0..=100) <= 80 {
                        return false;
                    }
                }
                true
            },
            Self::Tree => {
                if connecting_tiles.contains(&Self::Tree) == false {
                if thread_rng().gen_range(0..100) <= 50 {
                    return false;
                }
                }
                for tile in connecting_tiles {
                    if [Self::Grass, Self::Tree, Self::None, Self::Invalid].contains(tile) == false {
                        return false;
                    }
                }
                true
            }
            _ => false
        }
    }

    fn get_base_weight(&self) -> f32 {
        match self {
            Self::House => 0.1,
            Self::Water => 0.5,
            Self::Tree => 0.5,
            Self::Sand => 0.2,
            _ => 1.0
        }
    }

    fn init_domain_weights() -> HashMap<Self,f32> {
        let mut domain = HashMap::new();
        for tile in Self::DEFAULT_DOMAINS {
            domain.insert(tile,tile.get_base_weight()); //ToDo: replace magic number with get_weight function
        }
        domain
    }

    fn init_domain() -> HashMap<Self, bool> {
        let mut domain = HashMap::new();
        for tile in Self::DEFAULT_DOMAINS {
            domain.insert(tile, true);
        }
        domain
    }
}
