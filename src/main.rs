use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

use std::fs::File;
use std::io::prelude::*;

use bmp_rust::bmp::BMP;

fn main() {
    let mut tile_atlas = HashMap::new();
    //tile_atlas.insert((0,0), TileType::Water);
    for x in 0..128 {
        for y in 0..72 {
            //if (x,y) == (0,0) { continue }
            tile_atlas.insert((x,y), collapse_tile(&tile_atlas, x, y));
        }
    }
    //for tile in tile_atlas.clone() {
    //    tile_atlas.insert(tile.0, collapse_tile(&tile_atlas, tile.0.0, tile.0.1));
    //}
    //tile_atlas.insert((0,0), collapse_tile(&tile_atlas, 0, 0));
    let mut file = File::create("map.txt").unwrap();
    let mut string = String::new();
    let mut img = BMP::new(720, 1280, None);
    for x in 0..128 {
        for y in 0..72 {
            string.push(match tile_atlas.get(&(x as i32,y as i32)).unwrap() {
                TileType::Grass => 'g',
                TileType::Sand => 's',
                TileType::Tree => 't',
                TileType::Water => 'w',
                TileType::House => 'h',
                TileType::None => ' ',
                TileType::Invalid => '?',
           });
            let color: [u8; 4] = match tile_atlas.get(&(x as i32, y as i32)).unwrap() {
                TileType::Grass => [0,255,0,255],
                TileType::Water => [0,0,255,255],
                TileType::Sand => [255,255,0,255],
                TileType::Tree => [92,108,0,255],
                TileType::House => [245,40,145,255],
                _ => [0,0,0,255]
            };
            img.draw_rectangle(Some(color), Some(color), [(x*10) as u16, (y*10 as u16)], [((x+1)*10-1) as u16, ((y+1)*10-1) as u16]);
        }
        string.push('\n');
    }
    file.write_all(string.as_bytes()).unwrap();
    let _ = img.save_to_new("img.bmp");
}


struct Tile<const DOMAIN_SIZE: usize> {
    tile_type: TileType,
    x_pos: i32,
    y_pos: i32,
    connecting_tiles: [Option<TileType>; DOMAIN_SIZE]
}

impl<const DOMAIN_SIZE: usize> Tile<DOMAIN_SIZE> {
    fn new() -> Tile<DOMAIN_SIZE> {
       todo!(); 
    }
    /*
    fn collapse(&self, tile_atlas: &HashMap<(i32,i32), TileType>) -> TileType {
        let domain = self.get_domain(tile_atlas); 
    }
    */

    /*
    fn update_connecting_tiles(&self, tile_atlas: &HashMap<(i32,i32), TileType>) {
        let connecting_tile_positions = [(self.x_pos-1,self.y_pos),(self.x_pos,self.y_pos-1),(self.x_pos,self.y_pos+1),(self.x_pos+1,self.y_pos)];
        for tile in connecting_tile_positions {
            match tile_atlas.get(&tile) {
                Some(tile_type)
                None()
            }
        }
    }
    */

    fn get_domain(&self, tile_atlas: &HashMap<(i32,i32),TileType>) -> Vec<TileType> {
        let mut domain = TileType::init_domain();
        let mut domain_keys = vec![];
        for (key,_) in domain.clone() {
            domain_keys.push(key);
        }
        let connecting_tiles = [(self.x_pos-1,self.y_pos),(self.x_pos,self.y_pos-1),(self.x_pos,self.y_pos+1),(self.x_pos+1,self.y_pos)];
        for tile in connecting_tiles {
            if let Some(tile_type) = tile_atlas.get(&tile) {
                for domain_key in domain_keys.clone() {
                    if domain.get(&domain_key) == Some(&true) && tile_type.get_domain().contains(&domain_key) == false {
                        domain.insert(domain_key, false);
                    }
                }
            }
        }
        let mut filtered_domain = domain.into_iter().filter(|(key,value)| (value == &true)).map(|(tile_type, _)| tile_type).collect::<Vec<TileType>>();
        filtered_domain
    }

    fn get_entropy(&self, tile_atlas: &HashMap<(i32,i32),TileType>) -> i32 {
        self.get_domain(tile_atlas).len() as i32
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
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
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

    fn init_domain() -> HashMap<Self, bool> {
        let mut domain = HashMap::new();
        for tile in Self::DEFAULT_DOMAINS {
            domain.insert(tile, true);
        }
        domain
    }
}
