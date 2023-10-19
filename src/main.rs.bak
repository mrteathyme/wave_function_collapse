#![allow(unused_variables)]
use std::ops::Add;
use std::rc::Rc;

use std::collections::HashMap;

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::thread_rng;

use bmp_rust::bmp::BMP;

const DIRECTIONS: [Position; 8] = [
    Position(-1, 1, 0),
    Position(0, 1, 0),
    Position(1, 1, 0),
    Position(-1, 0, 0),
    Position(1, 0, 0),
    Position(-1, -1, 0),
    Position(0, -1, 0),
    Position(1, -1, 0),
];

//static DIRECTIONS: [Position;4] = [Position(-1,0,0), Position(0,1,0), Position(1,0,0), Position(0,-1,0)];

static RESOLUTION: (i32, i32) = (1280, 720);

fn main() {
    let mut rng = thread_rng();
    let mut tile_atlas: HashMap<Position, Rc<Tile<{ DIRECTIONS.len() }>>> = HashMap::new();
    let mut pending_tiles: HashMap<Position, Rc<Tile<{ DIRECTIONS.len() }>>> = HashMap::new();
    for x in 0..(RESOLUTION.0/10) {
        for y in 0..(RESOLUTION.1/10) {
            let tile = Rc::new(Tile::new(Position(x, y, 0)));
            tile_atlas.insert(Position(x, y, 0), tile);
        }
    }
    let mut lowest_tile = tile_atlas.get(&Position(rng.gen_range(0..(RESOLUTION.0/10)), rng.gen_range(0..(RESOLUTION.1/10)), 0)).unwrap().as_ref().clone();
    pending_tiles.insert(lowest_tile.position, Rc::new(lowest_tile.clone()));
    let mut iterations = 0;
    //Appeasing the compiler gods below lol
    //let mut lowest_tile: Tile<{ DIRECTIONS.len() }> = pending_tiles.get(&Position(rng.gen_range(0..(RESOLUTION.0/10)), rng.gen_range(0..(RESOLUTION.1/10)), 0)).unwrap().as_ref().clone();
    let mut previous_tile: Tile<{ DIRECTIONS.len() }> = Tile::new(Position(0, 0, -1));
    while pending_tiles.len() > 0 {
        let mut lowest_entropy = f32::MAX;
        for tile in pending_tiles.iter_mut() {
            if tile.1.get_entropy(&tile_atlas) <= lowest_entropy {
                lowest_entropy = tile.1.get_entropy(&tile_atlas);
                lowest_tile = tile.1.as_ref().clone();
            }
        }
        if lowest_tile == previous_tile {
            break;
        }
        lowest_tile.collapse(&mut tile_atlas, &mut pending_tiles);
        pending_tiles.remove(&lowest_tile.position);
        println!(
            "iteration: {iterations}, collapsed tile at position {:?}, result of collapse: {:?}",
            lowest_tile.position, lowest_tile.tile_type
        );

        tile_atlas.insert(lowest_tile.position, Rc::new(lowest_tile.clone()));
        previous_tile = lowest_tile.clone();
        iterations += 1;
    }

    let mut img = BMP::new(RESOLUTION.1, RESOLUTION.0 as u32, None);
    for x in 0..(RESOLUTION.0/10) {
        for y in 0..(RESOLUTION.1/10) {
            let color: [u8; 4] = match tile_atlas.get(&Position(x, y, 0)).unwrap().tile_type {
                TileType::Grass => [0, 255, 0, 255],
                TileType::Water => [0, 179, 255, 255],
                TileType::DeepWater => [0,0,255,255],
                TileType::BigTree => [21,76,0,255],
                TileType::Sand => [255, 255, 0, 255],
                TileType::Tree => [92, 108, 0, 255],
                TileType::House => [245, 40, 145, 255],
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
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct Position(i32, i32, i32);

impl Add for Position {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            0: self.0 + other.0,
            1: self.1 + other.1,
            2: self.2 + other.2,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Tile<const DOMAINSIZE: usize> {
    tile_type: TileType,
    position: Position,
    connecting_tiles: [Option<Rc<Self>>; DOMAINSIZE],
}

impl<const DOMAINSIZE: usize> Tile<DOMAINSIZE> {
    fn new(position: Position) -> Tile<DOMAINSIZE> {
        Tile {
            tile_type: TileType::None,
            position,
            connecting_tiles: std::array::from_fn(|_| None)
        }
    }

    fn collapse(
        &mut self,
        tile_atlas: &mut HashMap<Position, Rc<Tile<DOMAINSIZE>>>,
        pending_tiles: &mut HashMap<Position, Rc<Tile<DOMAINSIZE>>>,
    ) {
        self.update_connecting_tiles(tile_atlas);
        let mut choices = vec![];
        let mut weights = vec![];
        for (key, value) in self.get_domain_weights(tile_atlas) {
            if value > 0.0 {
                choices.push(key);
                weights.push(value);
            }
        }
        if weights.len() == 0 {
            self.tile_type = TileType::Invalid;
            return;
        }
        let dist = WeightedIndex::new(&weights).unwrap();
        let mut rng = thread_rng();
        self.tile_type = choices[dist.sample(&mut rng)];
        let temp_atlas = tile_atlas.clone();
        for (i, _) in self.connecting_tiles.clone().into_iter().enumerate() {
            let real_tile = tile_atlas.get_mut(&(self.position + DIRECTIONS[i]));
            if let Some(tile_inner) = real_tile {
                if tile_inner.tile_type == TileType::None {
                    let mut tile_clone = tile_inner.as_ref().clone();
                    tile_clone.update_connecting_tiles(&temp_atlas);
                    pending_tiles.insert(tile_inner.position, Rc::new(tile_clone));
                }
            }
        }
    }

    fn update_connecting_tiles(&mut self, tile_atlas: &HashMap<Position, Rc<Tile<DOMAINSIZE>>>) {
        for (i, direction) in DIRECTIONS.into_iter().enumerate() {
            self.connecting_tiles[i] = match tile_atlas.get(&(direction + self.position)) {
                Some(tile) => Some(tile.clone()),
                None => None,
            };
        }
    }

    fn get_domain_weights(
        &self,
        tile_atlas: &HashMap<Position, Rc<Tile<DOMAINSIZE>>>,
    ) -> HashMap<TileType, f32> {
        let mut weights = TileType::init_domain_weights();
        //let real_tiles: Rc<[Self]> = self.connecting_tiles.clone().into_iter().filter(|&value| value != &None).map(|value| value.unwrap()).collect();
        for (tile, weight) in weights.clone() {
            if tile.can_spawn(&self.connecting_tiles, tile_atlas, 0, self) == false {
                weights.insert(tile, 0.0);
                continue;
            }
            weights.insert(tile, weight * tile.get_weighting_multiplier(&self.connecting_tiles, tile_atlas));
        }
        weights
    }

    fn get_entropy(&self, tile_atlas: &HashMap<Position, Rc<Tile<DOMAINSIZE>>>) -> f32 {
        let mut weighting_sum: f32 = 0.0;
        let mut log_weights_sum: f32 = 0.0;
        for (_, weight) in self.get_domain_weights(tile_atlas) {
            if weight > 0.0 {
                weighting_sum += weight;
                log_weights_sum += weight * weight.log2();
            }
        }
        if weighting_sum == 0.0 {return f32::MAX};
        weighting_sum.log2() - (log_weights_sum / weighting_sum)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Copy)]
enum TileType {
    Grass,
    Water,
    Sand,
    Tree,
    House,
    DeepWater,
    BigTree,
    None,
    Invalid,
}

trait TileVariant<const DOMAINSIZE: usize> {
    const DOMAIN: [TileType; DOMAINSIZE];
    const VARIANT: TileType;

    fn neighbour_ok(tile_type: TileType) -> bool {
        if Self::DOMAIN.contains(&tile_type) {return true;}
        false
    }

    fn inverted_spawn<const TILENUM: usize>(
        proposed_tile_type: TileType,
        position: Position,
        atlas: &HashMap<Position, Rc<Tile<TILENUM>>>,
        new_center: Tile<TILENUM>
    ) -> bool {
        let mut rebuilt_tiles: [Option<Rc<Tile<TILENUM>>>; TILENUM] = std::array::from_fn(|_| None);
        let iterations = 0;
        for tile in new_center.connecting_tiles.clone() {
            match tile {
                Some(tile_inner) => {
                   let tile_type = if tile_inner.position == position {proposed_tile_type} else {tile_inner.tile_type};
                   rebuilt_tiles[iterations] = Some(Rc::new(Tile {position: tile_inner.position, tile_type, connecting_tiles: tile_inner.connecting_tiles.clone()}));  
                },
                None => continue
            }
        }
        proposed_tile_type.can_spawn(&rebuilt_tiles, atlas, 0, &new_center)
    }

    fn can_spawn<const TILENUM: usize>(
        connecting_tiles: &[Option<Rc<Tile<TILENUM>>>],
        atlas: &HashMap<Position, Rc<Tile<TILENUM>>>,
        recursion_level: i32,
        center: &Tile<TILENUM>
    ) -> bool {
        for tile in connecting_tiles {
            match tile {
                Some(tile_inner) => {
                    if tile_inner.tile_type.neighbour_ok(Self::VARIANT) == false
                    {
                        return false;
                    }
                    if tile_inner.tile_type.inverted_spawn(Self::VARIANT, center.position,atlas,  tile_inner.as_ref().clone()) == false { return false; }
                    //if tile_inner.tile_type.can_spawn(&[Some(Rc::new(Tile {position: center.position, tile_type: Self::VARIANT, connecting_tiles: center.connecting_tiles.clone()}))], atlas, recursion_level, center) == false { return false};
                }
                None => continue,
            }
        }
        true
    }
    fn get_weighting_multiplier<const TILENUM: usize>(
        connecting_tiles: &[Option<Rc<Tile<TILENUM>>>],
        tile_atlas: &HashMap<Position, Rc<Tile<TILENUM>>>,
    ) -> f32 {
        1.0
    }
}

struct Grass;
struct Water;
struct Sand;
struct Tree;
struct House;
struct DeepWater;
struct BigTree;

impl TileVariant<4> for BigTree {
    const DOMAIN: [TileType; 4] = [
        TileType::Tree,
        TileType::BigTree,
        TileType::None,
        TileType::Invalid
    ];
    const VARIANT: TileType = TileType::BigTree;

fn can_spawn<const TILENUM: usize>(
            connecting_tiles: &[Option<Rc<Tile<TILENUM>>>],
            atlas: &HashMap<Position, Rc<Tile<TILENUM>>>,
            recursion_level: i32,
            center: &Tile<TILENUM>
        ) -> bool {
        let mut tree_count = 0;
        let mut big_tree_count = 0;
        for tile in connecting_tiles {
            match tile {
                Some(tile_inner) => {
                    if tile_inner.tile_type.neighbour_ok(Self::VARIANT) == false { return false; }
                    if tile_inner.tile_type.inverted_spawn(Self::VARIANT, center.position,atlas,  tile_inner.as_ref().clone()) == false { return false; }
                    if tile_inner.tile_type == TileType::Tree {tree_count += 1}
                    if tile_inner.tile_type == TileType::BigTree {big_tree_count += 1}
                }
                None => continue,
            }
        }
        if tree_count < 3 && big_tree_count < 1 {
            return false
        }
        true
    }
}

impl TileVariant<4> for DeepWater {
    const DOMAIN: [TileType; 4] = [
        TileType::Water,
        TileType::DeepWater,
        TileType::None,
        TileType::Invalid,
    ];
    const VARIANT: TileType = TileType::DeepWater;

    fn can_spawn<const TILENUM: usize>(
            connecting_tiles: &[Option<Rc<Tile<TILENUM>>>],
            atlas: &HashMap<Position, Rc<Tile<TILENUM>>>,
            recursion_level: i32,
            center: &Tile<TILENUM>
        ) -> bool {
        let mut water_count = 0;
        let mut deep_water_count = 0;
        for tile in connecting_tiles {
            match tile {
                Some(tile_inner) => {
                    if tile_inner.tile_type.neighbour_ok(Self::VARIANT) == false { return false; }
                    if tile_inner.tile_type.inverted_spawn(Self::VARIANT, center.position,atlas,  tile_inner.as_ref().clone()) == false { return false; }
                    if tile_inner.tile_type == TileType::Water {water_count += 1}
                    if tile_inner.tile_type == TileType::DeepWater {deep_water_count += 1}
                }
                None => continue,
            }
        }
        if water_count < 3 && deep_water_count < 1 {
            return false
        }
        true
    }
}

impl TileVariant<3> for House {
    const DOMAIN: [TileType; 3] = [TileType::Grass, TileType::None, TileType::Invalid];
    const VARIANT: TileType = TileType::House;
}

impl TileVariant<5> for Sand {
    const DOMAIN: [TileType; 5] = [
        TileType::Grass,
        TileType::Sand,
        TileType::Water,
        TileType::Invalid,
        TileType::None,
    ];
    const VARIANT: TileType = TileType::Sand;

    fn can_spawn<const TILENUM: usize>(
        connecting_tiles: &[Option<Rc<Tile<TILENUM>>>],
        atlas: &HashMap<Position, Rc<Tile<TILENUM>>>,
        recursion_level: i32,center: &Tile<TILENUM>

    ) -> bool {
        let mut grass_count = 0;
        let mut water_count = 0;
        for tile in connecting_tiles {
            match tile {
                Some(tile_inner) => {
                    if tile_inner.tile_type.neighbour_ok(Self::VARIANT) == false { return false; }
                    if tile_inner.tile_type.inverted_spawn(Self::VARIANT, center.position,atlas,  tile_inner.as_ref().clone()) == false { return false; }
                    if tile_inner.tile_type == TileType::Grass {grass_count+=1; if grass_count > 3 {return false}}
                    if tile_inner.tile_type == TileType::Water {water_count+=1; if water_count > 3 {return false}}
                    for sub_tile in &tile_inner.connecting_tiles {
                        match sub_tile {
                            Some(sub_tile_inner) => {
                               if tile_inner.tile_type == TileType::Grass && (sub_tile_inner.tile_type == TileType::Water || sub_tile_inner.tile_type == TileType::None) { return true }
                                if tile_inner.tile_type == TileType::Water && (sub_tile_inner.tile_type == TileType::Grass || sub_tile_inner.tile_type == TileType::None) { return true }
                                //if tile_inner.tile_type == TileType::Sand && (sub_tile_inner.tile_type == TileType::Water || sub_tile_inner.tile_type == TileType::Sand || sub_tile_inner.tile_type == TileType::Grass || sub_tile_inner.tile_type == TileType::None) {return true}
                            },
                            None => continue
                        }
                    }
                }
                None => continue,
            }
        }
        false
    }

}

impl TileVariant<5> for Water {
    const DOMAIN: [TileType; 5] = [
        TileType::Sand,
        TileType::Water,
        TileType::DeepWater,
        TileType::None,
        TileType::Invalid,
    ];
    const VARIANT: TileType = TileType::Water;
}

impl TileVariant<6> for Grass {
    const DOMAIN: [TileType; 6] = [
        TileType::Grass,
        TileType::Tree,
        TileType::House,
        TileType::Sand,
        TileType::None,
        TileType::Invalid,
    ];
    const VARIANT: TileType = TileType::Grass;
}

impl TileVariant<5> for Tree {
    const DOMAIN: [TileType; 5] = [
        TileType::Grass,
        TileType::Tree,
        TileType::BigTree,
        TileType::None,
        TileType::Invalid,
    ];
    const VARIANT: TileType = TileType::Tree;
}

impl TileType {
    const DEFAULT_DOMAINS: [TileType; 7] = [
        Self::Grass,
        Self::Water,
        Self::Sand,
        Self::Tree,
        Self::House,
        Self::DeepWater,
        Self::BigTree
    ];

    fn inverted_spawn<const DOMAINSIZE: usize>(&self, tile_type: Self, position: Position,atlas: &HashMap<Position, Rc<Tile<DOMAINSIZE>>>, center: Tile<DOMAINSIZE>) -> bool {
        match self {
            Self::House => House::inverted_spawn(tile_type, position, atlas, center),
            Self::Tree => Tree::inverted_spawn(tile_type, position,atlas,  center),
            Self::Sand => Sand::inverted_spawn(tile_type, position,atlas,  center),
            Self::Grass => Grass::inverted_spawn(tile_type, position,atlas,  center),
            Self::Water => Water::inverted_spawn(tile_type, position,atlas,  center),
            Self::DeepWater => DeepWater::inverted_spawn(tile_type, position,atlas,  center),
            Self::BigTree => BigTree::inverted_spawn(tile_type, position,atlas,  center),
            _ => true
        }
    }

    fn neighbour_ok(&self, tile_type: TileType) -> bool {
        match self {
            Self::House => House::neighbour_ok(tile_type),
            Self::Tree => Tree::neighbour_ok(tile_type),
            Self::Sand => Sand::neighbour_ok(tile_type),
            Self::Grass => Grass::neighbour_ok(tile_type),
            Self::Water => Water::neighbour_ok(tile_type),
            Self::DeepWater => DeepWater::neighbour_ok(tile_type),
            Self::BigTree => BigTree::neighbour_ok(tile_type),
            _ => true,
        }

    }

    fn get_weighting_multiplier<const DOMAINSIZE: usize>(&self, connecting_tiles: &[Option<Rc<Tile<DOMAINSIZE>>>], atlas: &HashMap<Position, Rc<Tile<DOMAINSIZE>>>) -> f32 {
        match self {
            Self::House => House::get_weighting_multiplier(connecting_tiles,atlas),
            Self::Tree => Tree::get_weighting_multiplier(connecting_tiles,atlas),
            Self::Sand => Sand::get_weighting_multiplier(connecting_tiles,atlas),
            Self::Grass => Grass::get_weighting_multiplier(connecting_tiles,atlas),
            Self::Water => Water::get_weighting_multiplier(connecting_tiles,atlas),
            Self::DeepWater => DeepWater::get_weighting_multiplier(connecting_tiles,atlas),
            Self::BigTree => BigTree::get_weighting_multiplier(connecting_tiles,atlas),
            _ => 1.0,
        }
    }

    fn can_spawn<const DOMAINSIZE: usize>(
        &self,
        connecting_tiles: &[Option<Rc<Tile<DOMAINSIZE>>>],
        atlas: &HashMap<Position, Rc<Tile<DOMAINSIZE>>>,
        recursion_level: i32,center: &Tile<DOMAINSIZE>

    ) -> bool {
        match self {
            Self::House => House::can_spawn(connecting_tiles, atlas, recursion_level,center),
            Self::Grass => Grass::can_spawn(connecting_tiles, atlas, recursion_level,center),
            Self::Sand => Sand::can_spawn(connecting_tiles, atlas, recursion_level,center),
            Self::Water => Water::can_spawn(connecting_tiles, atlas, recursion_level,center),
            Self::DeepWater => DeepWater::can_spawn(connecting_tiles, atlas, recursion_level,center),
            Self::Tree => Tree::can_spawn(connecting_tiles, atlas, recursion_level,center),
            Self::BigTree => BigTree::can_spawn(connecting_tiles, atlas, recursion_level, center),
            _ => false,
        }
    }

    fn get_base_weight(&self) -> f32 {
        match self {
            Self::House => 0.1,
            Self::Water => 0.5,
            Self::Tree => 0.5,
            Self::Sand => 10.0,
            Self::DeepWater => 0.1,
            Self::BigTree => 0.1,
            Self::Grass => 0.5,
            _ => 1.0,
        }
    }

    fn init_domain_weights() -> HashMap<Self, f32> {
        let mut domain = HashMap::new();
        for tile in Self::DEFAULT_DOMAINS {
            domain.insert(tile, tile.get_base_weight()); //ToDo: replace magic number with get_weight function
        }
        domain
    }
}
