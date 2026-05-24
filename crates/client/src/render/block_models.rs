use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Clone, Resource)]
pub struct BlockRegistry {
    pub models: Vec<BlockModel>,
    pub id_to_index: HashMap<u16, usize>,
    pub textures: Vec<&'static str>,
}

#[derive(Clone)]
pub struct BlockModel {
    pub faces: [BlockFace; 6],
    #[allow(dead_code)]
    pub overlay: Option<OverlayFace>,
    #[allow(dead_code)]
    pub transparent: bool,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct OverlayFace {
    pub texture: usize,
    pub side_only: bool,
}

#[derive(Clone, Copy)]
pub struct BlockFace {
    pub texture: usize,
}

pub const DOWN: usize = 0;
pub const UP: usize = 1;
pub const NORTH: usize = 2;
pub const SOUTH: usize = 3;
pub const WEST: usize = 4;
pub const EAST: usize = 5;

fn cube_all(tex: usize) -> BlockModel {
    BlockModel {
        faces: [BlockFace { texture: tex }; 6],
        overlay: None,
        transparent: false,
    }
}

fn cube_bottom_top(bottom: usize, top: usize, side: usize) -> BlockModel {
    BlockModel {
        faces: [
            BlockFace { texture: bottom }, // down
            BlockFace { texture: top },    // up
            BlockFace { texture: side },   // north
            BlockFace { texture: side },   // south
            BlockFace { texture: side },   // west
            BlockFace { texture: side },   // east
        ],
        overlay: None,
        transparent: false,
    }
}

fn cube_column(end: usize, side: usize) -> BlockModel {
    BlockModel {
        faces: [
            BlockFace { texture: end },   // down
            BlockFace { texture: end },   // up
            BlockFace { texture: side },  // north
            BlockFace { texture: side },  // south
            BlockFace { texture: side },  // west
            BlockFace { texture: side },  // east
        ],
        overlay: None,
        transparent: false,
    }
}

fn glass() -> BlockModel {
    let mut m = cube_all(0);
    m.transparent = true;
    m
}

impl BlockRegistry {
    pub fn new() -> Self {
        let textures = vec![
            // 0: placeholder
            "",
            // Stone-type
            "block/stone",
            "block/granite",
            "block/polished_granite",
            "block/diorite",
            "block/polished_diorite",
            "block/andesite",
            "block/polished_andesite",
            "block/cobblestone",
            "block/bedrock",
            "block/stone_bricks",
            "block/bricks",
            "block/obsidian",
            // Dirt / ground
            "block/dirt",
            "block/coarse_dirt",
            "block/grass_block_top",
            "block/grass_block_side",
            "block/grass_block_side_overlay",
            "block/podzol_top",
            "block/podzol_side",
            "block/sand",
            "block/red_sand",
            "block/gravel",
            "block/clay",
            "block/moss_block",
            "block/snow",
            "block/mycelium_top",
            "block/mycelium_side",
            // Wood / planks
            "block/oak_planks",
            "block/spruce_planks",
            "block/birch_planks",
            "block/jungle_planks",
            "block/acacia_planks",
            "block/cherry_planks",
            "block/dark_oak_planks",
            "block/pale_oak_planks",
            "block/mangrove_planks",
            "block/bamboo_planks",
            "block/bamboo_mosaic",
            // Logs
            "block/oak_log_top",
            "block/oak_log",
            "block/spruce_log_top",
            "block/spruce_log",
            "block/birch_log_top",
            "block/birch_log",
            "block/jungle_log_top",
            "block/jungle_log",
            "block/acacia_log_top",
            "block/acacia_log",
            "block/cherry_log_top",
            "block/cherry_log",
            "block/dark_oak_log_top",
            "block/dark_oak_log",
            "block/mangrove_log_top",
            "block/mangrove_log",
            "block/pale_oak_log_top",
            "block/pale_oak_log",
            "block/stripped_oak_log_top",
            "block/stripped_oak_log",
            "block/stripped_spruce_log_top",
            "block/stripped_spruce_log",
            // Leaves
            "block/oak_leaves",
            "block/spruce_leaves",
            "block/birch_leaves",
            // Ores / minerals
            "block/coal_ore",
            "block/iron_ore",
            "block/gold_ore",
            "block/nether_gold_ore",
            "block/copper_ore",
            "block/lapis_ore",
            "block/redstone_ore",
            "block/diamond_ore",
            "block/emerald_ore",
            "block/coal_block",
            "block/iron_block",
            "block/gold_block",
            "block/diamond_block",
            "block/emerald_block",
            "block/lapis_block",
            "block/redstone_block",
            "block/netherite_block",
            "block/copper_block",
            // Deepslate
            "block/deepslate_top",
            "block/deepslate",
            "block/cobbled_deepslate",
            "block/deepslate_coal_ore",
            "block/deepslate_iron_ore",
            "block/deepslate_copper_ore",
            "block/deepslate_gold_ore",
            "block/deepslate_diamond_ore",
            "block/deepslate_emerald_ore",
            "block/deepslate_redstone_ore",
            "block/deepslate_lapis_ore",
            "block/tuff",
            "block/calcite",
            "block/dripstone_block",
            "block/mossy_cobblestone",
            // Nether
            "block/netherrack",
            "block/soul_sand",
            "block/magma",
            // Sandstone
            "block/sandstone_top",
            "block/sandstone_bottom",
            "block/sandstone",
            "block/red_sandstone_top",
            "block/red_sandstone_bottom",
            "block/red_sandstone",
            // Ice / snow
            "block/ice",
            "block/packed_ice",
            "block/blue_ice",
            // Other
            "block/bookshelf",
            "block/crafting_table_top",
            "block/crafting_table_front",
            "block/crafting_table_side",
            "block/furnace_top",
            "block/furnace_front",
            "block/furnace_side",
            "block/terracotta",
            "block/white_terracotta",
            "block/sponge",
            "block/glass",
            "block/water_still",
            "block/lava_still",
            "block/bone_block_side",
            "block/bone_block_top",
            // Wool
            "block/white_wool",
            "block/orange_wool",
            "block/magenta_wool",
            "block/light_blue_wool",
            "block/yellow_wool",
            "block/lime_wool",
            "block/pink_wool",
            "block/gray_wool",
            "block/light_gray_wool",
            "block/cyan_wool",
            "block/purple_wool",
            "block/blue_wool",
            "block/brown_wool",
            "block/green_wool",
            "block/red_wool",
            "block/black_wool",
            // Terracotta colors
            "block/orange_terracotta",
            "block/magenta_terracotta",
            "block/light_blue_terracotta",
            "block/yellow_terracotta",
            "block/lime_terracotta",
            "block/pink_terracotta",
            "block/gray_terracotta",
            "block/light_gray_terracotta",
            "block/cyan_terracotta",
            "block/purple_terracotta",
            "block/blue_terracotta",
            "block/brown_terracotta",
            "block/green_terracotta",
            "block/red_terracotta",
            "block/black_terracotta",
            "block/mud",
        ];

        let mut id_to_index = HashMap::new();
        let mut models = Vec::new();

        macro_rules! id {
            ($id:expr, $model:expr) => {
                let idx = models.len();
                models.push($model);
                id_to_index.insert($id, idx);
            };
        }

        let s = |name: &str| -> usize {
            textures.iter().position(|&t| t == name).unwrap()
        };

        // Stone-type
        id!(1, cube_all(s("block/stone")));
        id!(2, cube_all(s("block/granite")));
        id!(3, cube_all(s("block/polished_granite")));
        id!(4, cube_all(s("block/diorite")));
        id!(5, cube_all(s("block/polished_diorite")));
        id!(6, cube_all(s("block/andesite")));
        id!(7, cube_all(s("block/polished_andesite")));
        id!(14, cube_all(s("block/cobblestone")));
        id!(85, cube_all(s("block/bedrock")));
        id!(6780, cube_all(s("block/stone_bricks")));
        id!(2139, cube_all(s("block/bricks")));
        id!(2400, cube_all(s("block/obsidian")));
        id!(2399, cube_all(s("block/mossy_cobblestone")));

        // Dirt / ground
        id!(9, BlockModel {
            faces: [
                BlockFace { texture: s("block/dirt") },           // down
                BlockFace { texture: s("block/grass_block_top") }, // up
                BlockFace { texture: s("block/grass_block_side") }, // north
                BlockFace { texture: s("block/grass_block_side") }, // south
                BlockFace { texture: s("block/grass_block_side") }, // west
                BlockFace { texture: s("block/grass_block_side") }, // east
            ],
            overlay: Some(OverlayFace {
                texture: s("block/grass_block_side_overlay"),
                side_only: true,
            }),
            transparent: false,
        });
        id!(10, cube_all(s("block/dirt")));
        id!(11, cube_all(s("block/coarse_dirt")));
        id!(12, cube_bottom_top(s("block/dirt"), s("block/podzol_top"), s("block/podzol_side")));
        id!(13, cube_bottom_top(s("block/dirt"), s("block/podzol_top"), s("block/podzol_side")));
        id!(118, cube_all(s("block/sand")));
        id!(123, cube_all(s("block/red_sand")));
        id!(124, cube_all(s("block/gravel")));
        id!(5977, cube_all(s("block/clay")));
        id!(25903, cube_all(s("block/moss_block")));
        id!(11633, cube_all(s("block/terracotta")));
        id!(10165, cube_all(s("block/white_terracotta")));

        // Ice / snow
        id!(5958, cube_all(s("block/ice")));
        id!(11635, cube_all(s("block/packed_ice")));
        id!(13964, cube_all(s("block/blue_ice")));
        id!(5959, cube_all(s("block/snow")));

        // Wood planks
        id!(15, cube_all(s("block/oak_planks")));
        id!(16, cube_all(s("block/spruce_planks")));
        id!(17, cube_all(s("block/birch_planks")));
        id!(18, cube_all(s("block/jungle_planks")));
        id!(19, cube_all(s("block/acacia_planks")));
        id!(20, cube_all(s("block/cherry_planks")));
        id!(21, cube_all(s("block/dark_oak_planks")));
        id!(25, cube_all(s("block/pale_oak_planks")));
        id!(26, cube_all(s("block/mangrove_planks")));
        id!(27, cube_all(s("block/bamboo_planks")));
        id!(28, cube_all(s("block/bamboo_mosaic")));

        // Logs
        id!(136, cube_column(s("block/oak_log_top"), s("block/oak_log")));
        id!(137, cube_column(s("block/oak_log_top"), s("block/oak_log")));
        id!(138, cube_column(s("block/oak_log_top"), s("block/oak_log")));
        id!(139, cube_column(s("block/spruce_log_top"), s("block/spruce_log")));
        id!(140, cube_column(s("block/spruce_log_top"), s("block/spruce_log")));
        id!(141, cube_column(s("block/spruce_log_top"), s("block/spruce_log")));
        id!(142, cube_column(s("block/birch_log_top"), s("block/birch_log")));
        id!(143, cube_column(s("block/birch_log_top"), s("block/birch_log")));
        id!(144, cube_column(s("block/birch_log_top"), s("block/birch_log")));
        id!(145, cube_column(s("block/jungle_log_top"), s("block/jungle_log")));
        id!(146, cube_column(s("block/jungle_log_top"), s("block/jungle_log")));
        id!(147, cube_column(s("block/jungle_log_top"), s("block/jungle_log")));
        id!(148, cube_column(s("block/acacia_log_top"), s("block/acacia_log")));
        id!(149, cube_column(s("block/acacia_log_top"), s("block/acacia_log")));
        id!(150, cube_column(s("block/acacia_log_top"), s("block/acacia_log")));
        id!(151, cube_column(s("block/cherry_log_top"), s("block/cherry_log")));
        id!(152, cube_column(s("block/cherry_log_top"), s("block/cherry_log")));
        id!(153, cube_column(s("block/cherry_log_top"), s("block/cherry_log")));
        id!(154, cube_column(s("block/dark_oak_log_top"), s("block/dark_oak_log")));
        id!(155, cube_column(s("block/dark_oak_log_top"), s("block/dark_oak_log")));
        id!(156, cube_column(s("block/dark_oak_log_top"), s("block/dark_oak_log")));
        id!(157, cube_column(s("block/pale_oak_log_top"), s("block/pale_oak_log")));
        id!(158, cube_column(s("block/pale_oak_log_top"), s("block/pale_oak_log")));
        id!(159, cube_column(s("block/pale_oak_log_top"), s("block/pale_oak_log")));
        id!(160, cube_column(s("block/mangrove_log_top"), s("block/mangrove_log")));
        id!(161, cube_column(s("block/mangrove_log_top"), s("block/mangrove_log")));
        id!(162, cube_column(s("block/mangrove_log_top"), s("block/mangrove_log")));

        // Stripped logs
        id!(192, cube_column(s("block/stripped_oak_log_top"), s("block/stripped_oak_log")));
        id!(193, cube_column(s("block/stripped_oak_log_top"), s("block/stripped_oak_log")));
        id!(194, cube_column(s("block/stripped_oak_log_top"), s("block/stripped_oak_log")));
        id!(171, cube_column(s("block/stripped_spruce_log_top"), s("block/stripped_spruce_log")));
        id!(172, cube_column(s("block/stripped_spruce_log_top"), s("block/stripped_spruce_log")));
        id!(173, cube_column(s("block/stripped_spruce_log_top"), s("block/stripped_spruce_log")));

        // Leaves (transparent)
        id!(278, glass());
        id!(306, glass());
        id!(334, glass());
        id!(362, glass());
        id!(390, glass());
        id!(418, glass());
        id!(446, glass());
        id!(502, glass());

        // Ores
        id!(129, cube_all(s("block/gold_ore")));
        id!(130, cube_bottom_top(s("block/deepslate_top"), s("block/deepslate_top"), s("block/deepslate_gold_ore")));
        id!(131, cube_all(s("block/iron_ore")));
        id!(132, cube_bottom_top(s("block/deepslate_top"), s("block/deepslate_top"), s("block/deepslate_iron_ore")));
        id!(133, cube_all(s("block/coal_ore")));
        id!(134, cube_bottom_top(s("block/deepslate_top"), s("block/deepslate_top"), s("block/deepslate_coal_ore")));
        id!(135, cube_all(s("block/nether_gold_ore")));
        id!(23970, cube_all(s("block/copper_ore")));

        // Deepslate
        id!(22109, cube_all(s("block/tuff")));
        id!(23344, cube_all(s("block/calcite")));
        id!(25796, cube_all(s("block/dripstone_block")));
        id!(25967, cube_all(s("block/cobbled_deepslate")));

        // Nether
        id!(6028, cube_all(s("block/netherrack")));
        id!(6029, cube_all(s("block/soul_sand")));
        id!(13566, cube_all(s("block/magma")));

        // Metals
        id!(2137, cube_all(s("block/gold_block")));
        id!(2138, cube_all(s("block/iron_block")));
        id!(4340, cube_all(s("block/diamond_block")));
        id!(8449, cube_all(s("block/emerald_block")));
        id!(565, cube_all(s("block/lapis_block")));
        id!(11634, cube_all(s("block/coal_block")));
        id!(20475, cube_all(s("block/netherite_block")));

        // Sandstone
        id!(578, cube_bottom_top(s("block/sandstone_bottom"), s("block/sandstone_top"), s("block/sandstone")));
        id!(579, cube_bottom_top(s("block/sandstone_bottom"), s("block/sandstone_top"), s("block/sandstone")));
        id!(580, cube_bottom_top(s("block/sandstone_bottom"), s("block/sandstone_top"), s("block/sandstone")));

        // Wool
        id!(2093, cube_all(s("block/white_wool")));
        id!(2094, cube_all(s("block/orange_wool")));
        id!(2095, cube_all(s("block/magenta_wool")));
        id!(2096, cube_all(s("block/light_blue_wool")));
        id!(2097, cube_all(s("block/yellow_wool")));
        id!(2098, cube_all(s("block/lime_wool")));
        id!(2099, cube_all(s("block/pink_wool")));
        id!(2100, cube_all(s("block/gray_wool")));
        id!(2101, cube_all(s("block/light_gray_wool")));
        id!(2102, cube_all(s("block/cyan_wool")));
        id!(2103, cube_all(s("block/purple_wool")));
        id!(2104, cube_all(s("block/blue_wool")));
        id!(2105, cube_all(s("block/brown_wool")));
        id!(2106, cube_all(s("block/green_wool")));
        id!(2107, cube_all(s("block/red_wool")));
        id!(2108, cube_all(s("block/black_wool")));

        // Other
        id!(560, cube_all(s("block/sponge")));
        id!(562, cube_all(s("block/glass")));
        id!(2142, cube_bottom_top(s("block/oak_planks"), s("block/oak_planks"), s("block/bookshelf")));
        id!(4341, BlockModel {
            faces: [
                BlockFace { texture: s("block/oak_planks") },                // down
                BlockFace { texture: s("block/crafting_table_top") },        // up
                BlockFace { texture: s("block/crafting_table_front") },      // north
                BlockFace { texture: s("block/crafting_table_side") },       // south
                BlockFace { texture: s("block/crafting_table_side") },       // west
                BlockFace { texture: s("block/crafting_table_front") },      // east
            ],
            overlay: None,
            transparent: false,
        });
        BlockRegistry { models, id_to_index, textures }
    }

    pub fn get(&self, id: u16) -> Option<&BlockModel> {
        self.id_to_index.get(&id).map(|&i| &self.models[i])
    }
}
