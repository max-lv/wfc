
use crate::wfc::{WfcTile};

pub fn pipes() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("./pipes_tileset.png");

    let mut tiles = Vec::new();
    tiles.push(WfcTile {
        index: 4*0 + 0, // col: 0, row: 0,
        connection_types: [1,1,0,1,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 4*0 + 2, // col: 2, row: 0,
        connection_types: [0,0,0,0,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 4*1 + 0, // col: 0, row: 1,
        connection_types: [0,1,0,1,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 4*0 + 1, // col: 1, row: 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 4*1 + 1, // col: 1, row: 1,
        connection_types: [0,0,1,1,0,0],
        angle: 0,
    });
    // connecting pipe
    tiles.push(WfcTile {
        index: 4*1 + 2, // col: 2, row: 1,
        connection_types: [0,1,0,2,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 4*0 + 3, // col: 3, row: 0,
        connection_types: [0,0,2,2,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 4*1 + 3, // col: 3, row: 1,
        connection_types: [0,2,0,2,0,0],
        angle: 0,
    });
    return (tilemap, tiles, 4);
}

pub fn flat_city() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("./flat-city.png");

    let mut tiles = Vec::new();
    tiles.push(WfcTile {
        index: 8*0 + 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 1,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 2,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 3,
        connection_types: [1,1,0,0,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 4,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    // walls
    tiles.push(WfcTile {
        index: 8*1 + 0,
        connection_types: [1,2,1,2,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*1 + 1,
        connection_types: [1,1,2,2,0,0],
        angle: 0,
    });
    // fat blocks
    tiles.push(WfcTile {
        index: 8*1 + 2,
        connection_types: [1,1,3,3,0,0],
        angle: 0,
    });
//    tiles.push(WfcTile {
//        col: 3,
//        row: 1,
//        connection_types: [1,1,3,3,0,0],
//        angle: 0,
//    });
    tiles.push(WfcTile {
        index: 8*1 + 4,
        connection_types: [3,3,3,3,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 5,
        connection_types: [1,0,1,1,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*1 + 5,
        connection_types: [1,0,0,0,0,0],
        angle: 0,
    });

    return (tilemap, tiles, 8);
}

pub fn flat_city_paths_only() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("./flat-city.png");

    let mut tiles = Vec::new();
    tiles.push(WfcTile {
        index: 8*0 + 0,
        connection_types: [1,1,1,1,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 2,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 3,
        connection_types: [1,1,0,0,0,0],
        angle: 0,
    });
    tiles.push(WfcTile {
        index: 8*0 + 5,
        connection_types: [1,0,1,1,0,0],
        angle: 0,
    });

    return (tilemap, tiles, 8);
}

pub fn stairs_3d() -> (String, Vec<WfcTile>, u32) {
    let tilemap = String::from("");

    let mut tiles = Vec::new();
    // empty
    tiles.push(WfcTile {
        index: 0,
        connection_types: [0,0,0,0,0,0],
        angle: 0,
    });
    // stairs empty
    tiles.push(WfcTile {
        index: 5,
        connection_types: [0,0,1,0,0,2],
        angle: 0,
    });
    // stairs
    tiles.push(WfcTile {
        index: 1,
        connection_types: [1,0,0,0,2,0],
        angle: 0,
    });
    // line
    tiles.push(WfcTile {
        index: 2,
        connection_types: [1,0,1,0,0,0],
        angle: 0,
    });
    // T-junction
    tiles.push(WfcTile {
        index: 3,
        connection_types: [1,1,1,0,0,0],
        angle: 0,
    });
    // deadend
    tiles.push(WfcTile {
        index: 4,
        connection_types: [1,0,0,0,0,0],
        angle: 0,
    });
//    // debug
//    tiles.push(WfcTile {
//        index: 5,
//        connection_types: [0,0,0,0,3,2],
//        angle: 0,
//    });
    return (tilemap, tiles, 0);
}

