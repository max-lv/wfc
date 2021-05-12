
use std::collections::{HashSet};
use rand::Rng;
use rand::SeedableRng;

/*
worldmap
- will always be container of tiles
- have to be indexed by (arbitrary?) index
  x,y for 2d (tiles, triangles, hexes)
  x,y,z for 3d
  I want to index it in order
- directions
  2d has: 3 (trinagles), 4 (tiles), 6 (hexes)
  - WFC and WFC_tile should be generic over directions. This will
    bring type safety for my tilesets a.k.a saves me headache of checking if all tiles in tileset have neccesery directions.
- I don't care about tiles or their rotation, I just need Vec<tile> and tile must have directions.
  main task I'm solving: given 'two stacks of tiles and direction' remove all non-compatible tiles.
*/

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct WfcTile {
    pub index: u32,
    pub connection_types: [usize; 6],
    pub angle: u32,
    pub is_rotatable: bool,
}

impl WfcTile {
    pub fn rotate(&mut self, rot: u32) -> &Self {
        self.angle = (self.angle + 90*rot) % 360;
        let rot = rot as usize;
        let arr = self.connection_types;

        // FIXME: up/down rotation is an ugly hack
        let mut up_dir = arr[4];
        if arr[4] != 0 {
            up_dir = arr[4] + (rot+1)*1000;
        }
        let mut down_dir = arr[5];
        if arr[5] != 0 {
            down_dir = arr[5] + (rot+1)*1000;
        }
        self.connection_types = [arr[rot % 4], arr[(rot+1) % 4], arr[(rot+2) % 4], arr[(rot+3) % 4], up_dir, down_dir];
        return self;
    }
}

impl PartialEq for WfcTile {
    fn eq(&self, other:&Self) -> bool {
        self.index == other.index && self.connection_types == other.connection_types && self.angle == other.angle
    }
}

fn choose_random<'a, Any>(rng: &mut rand::rngs::StdRng, vec: &'a Vec<Any>) -> &'a Any {
    &vec[rng.gen_range(0..vec.len())]
}

pub fn create_big_tile(gen_con: &mut usize, size: (usize, usize, usize), big_tile: Vec<Option<(u32, [usize;6])>>) -> Vec<WfcTile> {
    let mut wfc_big_tile = big_tile.iter().map(|tile|
        match tile {
            Some((idx, connections)) => Some(WfcTile {
                index: *idx,
                connection_types: *connections,
                angle: 0,
                is_rotatable: false,
            }),
            None => None,
        }).collect::<Vec<Option<WfcTile>>>();
    let (x_size, y_size, z_size) = size;

    for y in 0..y_size {
    for x in 0..x_size {
        let pos = y*x_size + x;
        let x = x as i32;
        let y = y as i32;

        if wfc_big_tile[pos] == None {
            continue;
        }

        // north
        if y-1 >= 0 && y-1 < y_size as i32 {
            let ppos = (y as usize -1)*x_size + x as usize;
            // generate connection
            if wfc_big_tile[ppos] != None {
                wfc_big_tile[ pos].as_mut().unwrap().connection_types[0] = *gen_con; // north
                wfc_big_tile[ppos].as_mut().unwrap().connection_types[2] = *gen_con; // south
                *gen_con += 1;
            }
        }

        // west
        if x-1 >= 0 && x-1 < x_size as i32 {
            let ppos = y as usize * x_size + x as usize - 1;
            // generate connection
            if wfc_big_tile[ppos] != None {
                wfc_big_tile[ pos].as_mut().unwrap().connection_types[1] = *gen_con; // west
                wfc_big_tile[ppos].as_mut().unwrap().connection_types[3] = *gen_con; // east
                *gen_con += 1;
            }
        }

        // up
        // TODO
    }
    }

    // add rotations
    // TODO: avoid duplicated rotations when tile is symmetrical.
    // TODO: something more elegant than multiplying connections by 1000, for example:
    //       have connection_types store tuple of (u16,u16), so user has u16 for his connections, and
    //       then I have u16 for internal connections.
    let wfc_big_tile: Vec<WfcTile> = wfc_big_tile.into_iter().flat_map(|x| x.into_iter()).collect();
    let mut rv = Vec::<WfcTile>::with_capacity(wfc_big_tile.len()*4);
    for _tile in &wfc_big_tile {
        for rot in 0..4 {
            let mut tile = _tile.clone();
            tile.rotate(rot as u32);
            for i in 0..6 {
                if tile.connection_types[i] >= 1000 {
                    tile.connection_types[i] *= rot+1;
                }
            }
            println!("{:?}", tile);
            rv.push(tile)
        }
    }
    return rv;
}

#[derive(Clone)]
pub struct Worldmap {
    pub values: Vec<Vec<WfcTile>>,
    pub size: [usize; 3],
    pub len: usize,
}

impl Worldmap {
    pub fn new3d(x: usize, y: usize, z: usize) -> Worldmap {
        Worldmap {
            values: vec![vec![]; x*y*z],
            size: [x, y, z],
            len: x*y*z,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn tmp_iter(&self) -> impl Iterator<Item = Position> {
        let mut values = Vec::<Position>::with_capacity(self.len);
        for x in 0..self.size[0] {
        for y in 0..self.size[1] {
        for z in 0..self.size[2] {
            values.push([x,y,z]);
        }
        }
        }
        return values.into_iter();
    }

    pub fn move_(&self, square: Position, dir: &Direction) -> Option<Position> {
        let (dx, dy, dz) = match dir {
            Direction::NORTH => ( 0, -1,  0),
            Direction::EAST  => (-1,  0,  0), // FIXME: ??? for some reason EAST and WEST switched around, 
            Direction::SOUTH => ( 0,  1,  0), //        now it should go counter-clockwise
            Direction::WEST  => ( 1,  0,  0),
            Direction::UP    => ( 0,  0,  1),
            Direction::DOWN  => ( 0,  0, -1),
        };
        let [XS,YS,ZS] = self.size;
        let [x, y, z] = square;
        let x = x as i32 + dx;
        let y = y as i32 + dy;
        let z = z as i32 + dz;
        if x < 0 || x as usize >= XS { return None; }
        if y < 0 || y as usize >= YS { return None; }
        if z < 0 || z as usize >= ZS { return None; }
        return Some([x as usize, y as usize, z as usize]);
    }
}

// !!WATCH YOUR STEP!! Rust Hell Below
// -----------------------------------
impl std::ops::Index<usize> for Worldmap {
    type Output = Vec<WfcTile>;
    fn index<'a>(&'a self, idx: usize) -> &'a Vec<WfcTile> {
        return &self.values[idx]
    }
}

impl std::ops::IndexMut<usize> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut Vec<WfcTile> {
        return &mut self.values[idx]
    }
}
impl std::ops::Index<(usize, usize)> for Worldmap {
    type Output = Vec<WfcTile>;
    fn index<'a>(&'a self, idx: (usize, usize)) -> &'a Vec<WfcTile> {
        let (x, y) = idx;
        return &self.values[x + y*self.size[0]]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: (usize, usize)) -> &'a mut Vec<WfcTile> {
        let (x, y) = idx;
        return &mut self.values[x + y*self.size[0]]
    }
}

impl std::ops::Index<(usize, usize, usize)> for Worldmap {
    type Output = Vec<WfcTile>;
    fn index<'a>(&'a self, idx: (usize, usize, usize)) -> &'a Vec<WfcTile> {
        let (x, y, z) = idx;
        return &self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}

impl std::ops::IndexMut<(usize, usize, usize)> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: (usize, usize, usize)) -> &'a mut Vec<WfcTile> {
        let (x, y, z) = idx;
        return &mut self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}

impl std::ops::Index<Position> for Worldmap {
    type Output = Vec<WfcTile>;
    fn index<'a>(&'a self, idx: Position) -> &'a Vec<WfcTile> {
        let [x, y, z] = idx;
        return &self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}

impl std::ops::IndexMut<Position> for Worldmap {
    fn index_mut<'a>(&'a mut self, idx: Position) -> &'a mut Vec<WfcTile> {
        let [x, y, z] = idx;
        return &mut self.values[x + y*self.size[0] + z*self.size[0]*self.size[1]]
    }
}


type Position = [usize; 3];
pub enum Direction {NORTH, EAST, SOUTH, WEST, UP, DOWN}

impl Direction {
    fn flip(&self) -> Direction {
        match self {
            Direction::NORTH => Direction::SOUTH,
            Direction::EAST  => Direction::WEST,
            Direction::UP    => Direction::DOWN,
            Direction::SOUTH => Direction::NORTH,
            Direction::WEST  => Direction::EAST,
            Direction::DOWN  => Direction::UP,
        }
    }
}

impl Into<usize> for Direction {
    fn into(self) -> usize {
        match self {
            Direction::NORTH => 0,
            Direction::EAST  => 1,
            Direction::SOUTH => 2,
            Direction::WEST  => 3,
            Direction::UP    => 4,
            Direction::DOWN  => 5,
        }
    }
}

pub struct WFC {
    pub tiles: Vec<WfcTile>,
    pub worldmap: Worldmap,
    pub seed: u64,
    rng: rand::rngs::StdRng,
}

impl WFC {
    // TODO: worldmap should hold indexes into tiles
    pub fn init(worldmap: Worldmap, tiles: Vec<WfcTile>, seed: u64) -> WFC {
        let mut wfc = WFC {
            tiles,
            worldmap,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            seed,
        };
        wfc.init_worldmap();
        wfc
    }

    pub fn init_rng(&mut self, seed: u64) {
        self.seed = seed;
        self.rng = rand::rngs::StdRng::seed_from_u64(seed);
    }

    fn init_tile(tiles: &Vec<WfcTile>, square: &mut Vec<WfcTile>) {
        square.clear();
        for tile in tiles {
            if !tile.is_rotatable {
                square.push(tile.clone());
                continue
            }

            // skip fully symmetrical tiles
            if tile.connection_types[0..4].iter().filter(|&x| *x != tile.connection_types[0]).collect::<Vec<&usize>>().len() == 0 {
                square.push(tile.clone());
                continue;
            }
            // TODO:
            // I-symmetry
            if tile.connection_types[0] == tile.connection_types[2] && tile.connection_types[1] == tile.connection_types [3] {
                square.push(*tile.clone().rotate(1));
                square.push(tile.clone());
                continue;
            }

            // All four rotations are needed
            // L-symmetry
            // T-symmetry
            for i in 0..4 {
                let mut new_wfc_tile = tile.clone();
                new_wfc_tile.rotate(i);
                square.push(new_wfc_tile);
            }
        }
    }

    pub fn init_worldmap(&mut self) {
        // fill worldmap with stuff
        for i in 0..self.worldmap.len {
            WFC::init_tile(&self.tiles, &mut self.worldmap[i]);
        }
    }

    #[allow(non_snake_case)]
    pub fn print_worldmap(&self) {
        // debug print worldmap
        let [XS,YS,ZS] = self.worldmap.size;
        for x in 0..XS {
            for y in 0..YS {
                for z in 0..ZS {
                    let tile = self.worldmap[(x,y,z)][0];
                    if tile.index == 0 { continue; }
                    print!("({}, {}, {}, {}, {}), ", x, y, z, tile.angle, tile.index);
                }
            }
            println!("");
        }
    }

    fn find_squares_in_order(&self) -> Vec::<Position> {
        let mut available_squares = Vec::<Position>::with_capacity(1);
        for square in self.worldmap.tmp_iter() {
            if self.worldmap[square].len() == 1 {
                continue;
            }
            available_squares.push(square);
            return available_squares;
        }

        // always empty:
        return available_squares;
    }

    pub fn add_tile(&mut self, square: Position, tile: WfcTile) -> Result<(), String> {
        self.worldmap[square].clear();
        self.worldmap[square].push(tile);

        self.propagate(square)?;

        return Ok(());
    }

    pub fn wfc_step(&mut self) -> Result<bool, String> {
        let mut worldmap_copy = self.worldmap.clone();

        loop {
            let square = match self.collapse() {
                Some(x) => x,
                None => return Ok(true),
            };

            match self.propagate(square) {
                Ok(_) => return Ok(false),
                Err(_) => {
                    // backtrack / error recovery

                    if worldmap_copy[square].len() == 1 {
                        return Err(format!("error: collapse of square {:?} resulted in an empty stack after trying all available tiles", square));
                    }

                    // remove selected tile from list of available
                    worldmap_copy[square].retain(|&x| x != self.worldmap[square][0]);
                    self.worldmap = worldmap_copy.clone();
                    println!("Backtracking for square {:?}", square);
                },
            };
        }
    }

    pub fn collapse(&mut self) -> Option<Position> {
        let available_squares = self.find_squares_in_order();

        if available_squares.len() == 0 {
            return None
        }
        let square = *choose_random(&mut self.rng, &available_squares);

        // observe
        let selected_tile = *choose_random(&mut self.rng, &self.worldmap[square]);

        //println!("selected_tile: {:?}  square: {:?}  stack: {:?}", selected_tile, square, self.worldmap[square].len());
        self.worldmap[square].clear();
        self.worldmap[square].push(selected_tile);
        return Some(square);
    }

    /// returns true if we changed available connections, false otherwise
    fn update_tile_stack(&mut self, connections: &HashSet<usize>, square: Position, dir: Direction) -> Result<bool, String> {
        // we are trying to access tile beyond edge
        let square = match self.worldmap.move_(square, &dir) {
            Some(x) => x,
            None => return Ok(false),
        };

        // this direction has all connections available
        // FIXME: 3 is hardcoded, but should be calculated.
        //        probably should put it into WFC struct.
        //        can be calculated from original tiles.
        // TODO: self.worldmap.max_connections(dir) -> usize
        //if connections.len() == 4 {
        //    return Ok(false);
        //}

        let mut ok_stack = Vec::<WfcTile>::with_capacity(5);

        let stack = &self.worldmap[square];
        for tile in stack {
            let d: usize = dir.flip().into();
            if !connections.contains(&tile.connection_types[d]) {
                continue
            }
            ok_stack.push(*tile);
        }
        if self.worldmap[square].len() == ok_stack.len() {
            //println!("  stack didn't change");
            return Ok(false);
        }
        if ok_stack.len() == 0 {
            //println!("  stack empty");
            return Err(format!("error: tile-stack reduced to 0!!!  tile: {:?}", square));
        }
        self.worldmap[square] = ok_stack;
//        println!("update_tile_stack has changed connections {:?}", map_square);
        //println!("  stack changed");
        return Ok(true);
    }

    // finds all available connections from this tile for each direction
    fn gather_available_connections(&self, square: Position) -> Vec::<std::collections::HashSet<usize>> {
        let mut connections = Vec::<std::collections::HashSet<usize>>::with_capacity(6);
        for _ in 0..6 {
            connections.push(std::collections::HashSet::<usize>::new());
        }
        let stack = &self.worldmap[square];
        for tile in stack {
            for i in 0..6 {
                connections[i].insert(tile.connection_types[i]);
            }
        }
        return connections;
    }

    // wfc3d: map_square probably can be anything
    pub fn propagate(&mut self, square: Position) -> Result<(), String> {
        let connections = self.gather_available_connections(square);
        //println!("propagate {:?} connections: {:?}", square, connections);

        // clear direction
        let is_recurse0 = self.update_tile_stack(&connections[0], square, Direction::NORTH)?;
        let is_recurse1 = self.update_tile_stack(&connections[1], square, Direction::EAST)?;
        let is_recurse2 = self.update_tile_stack(&connections[2], square, Direction::SOUTH)?;
        let is_recurse3 = self.update_tile_stack(&connections[3], square, Direction::WEST)?;
        let is_recurse4 = self.update_tile_stack(&connections[4], square, Direction::UP)?;
        let is_recurse5 = self.update_tile_stack(&connections[5], square, Direction::DOWN)?;

        // recurse in that direction
        if is_recurse0 { self.propagate(self.worldmap.move_(square, &Direction::NORTH).unwrap())?; }
        if is_recurse1 { self.propagate(self.worldmap.move_(square, &Direction::EAST).unwrap())?; }
        if is_recurse2 { self.propagate(self.worldmap.move_(square, &Direction::SOUTH).unwrap())?; }
        if is_recurse3 { self.propagate(self.worldmap.move_(square, &Direction::WEST).unwrap())?; }
        if is_recurse4 { self.propagate(self.worldmap.move_(square, &Direction::UP).unwrap())?; }
        if is_recurse5 { self.propagate(self.worldmap.move_(square, &Direction::DOWN).unwrap())?; }

        return Ok(());
    }

    // wfcstate: init, running, error, done
    // wfcstep should do propagation
    pub fn run(&mut self) -> Result<&Worldmap, String> {
        loop {
            let is_done = self.wfc_step()?;
            if is_done {
                return Ok(&self.worldmap);
            }
        }
    }

    pub fn run_until_success(&mut self) -> &Worldmap {
        // this is important, because worldmap might have been initialised by add_tile()
        let worldmap = self.worldmap.clone();
        loop {
            match self.run() {
                Ok(_) => return &self.worldmap,
                _ => (),
            };
            self.init_rng(self.seed + 1);
            self.worldmap = worldmap.clone();
        }
    }
}
