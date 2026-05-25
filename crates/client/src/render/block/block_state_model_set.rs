use std::collections::HashMap;

use crate::resources::model::sprite::material::Baked;

use super::dispatch::block_state_model::BlockStateModel;
use super::dispatch::single_variant::{BlockFace, OverlayFace, SingleVariant};
use ferrite_core::block::BlockState;

pub struct BlockStateModelSet {
    pub model_by_state: HashMap<BlockState, BlockStateModel>,
    pub missing_model: BlockStateModel,
    pub textures: Vec<&'static str>,
}

impl BlockStateModelSet {
    pub fn new(
        model_by_state: HashMap<BlockState, BlockStateModel>,
        missing_model: BlockStateModel,
        textures: Vec<&'static str>,
    ) -> Self {
        Self {
            model_by_state,
            missing_model,
            textures,
        }
    }

    pub fn get(&self, state: BlockState) -> &BlockStateModel {
        self.model_by_state
            .get(&state)
            .unwrap_or(&self.missing_model)
    }

    pub fn missing_model(&self) -> &BlockStateModel {
        &self.missing_model
    }

    pub fn get_particle_material(&self, state: BlockState) -> Baked {
        self.get(state).particle_material()
    }
}

/// Build the default hardcoded block state model set.
/// TODO: replace with JSON model loading.
pub fn build_default_block_state_model_set() -> BlockStateModelSet {
    let textures = vec![
        "",
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
        "block/oak_leaves",
        "block/spruce_leaves",
        "block/birch_leaves",
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
        "block/netherrack",
        "block/soul_sand",
        "block/magma",
        "block/sandstone_top",
        "block/sandstone_bottom",
        "block/sandstone",
        "block/red_sandstone_top",
        "block/red_sandstone_bottom",
        "block/red_sandstone",
        "block/ice",
        "block/packed_ice",
        "block/blue_ice",
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

    let model_tex = |name: &str| -> usize { textures.iter().position(|&t| t == name).unwrap() };

    fn cube(
        tex_names: &[&str],
        face_tex: [usize; 6],
        transparent: bool,
        overlay: Option<[usize; 2]>,
    ) -> SingleVariant {
        SingleVariant {
            faces: [
                BlockFace {
                    texture: face_tex[0],
                },
                BlockFace {
                    texture: face_tex[1],
                },
                BlockFace {
                    texture: face_tex[2],
                },
                BlockFace {
                    texture: face_tex[3],
                },
                BlockFace {
                    texture: face_tex[4],
                },
                BlockFace {
                    texture: face_tex[5],
                },
            ],
            overlay: overlay.map(|o| OverlayFace {
                texture: o[0],
                side_only: o[1] != 0,
            }),
            transparent,
            texture_names: tex_names.iter().map(|&s| s.to_string()).collect(),
            face_texture_names: face_tex,
        }
    }

    let mut model_by_state = HashMap::new();

    macro_rules! id {
        ($id:expr, $model:expr) => {
            model_by_state.insert(BlockState::from_raw($id), $model.into_model());
        };
    }

    let s = |name: &str| model_tex(name);

    id!(1, cube(&textures, [s("block/stone"); 6], false, None));
    id!(2, cube(&textures, [s("block/granite"); 6], false, None));
    id!(
        3,
        cube(&textures, [s("block/polished_granite"); 6], false, None)
    );
    id!(4, cube(&textures, [s("block/diorite"); 6], false, None));
    id!(
        5,
        cube(&textures, [s("block/polished_diorite"); 6], false, None)
    );
    id!(6, cube(&textures, [s("block/andesite"); 6], false, None));
    id!(
        7,
        cube(&textures, [s("block/polished_andesite"); 6], false, None)
    );
    id!(
        14,
        cube(&textures, [s("block/cobblestone"); 6], false, None)
    );
    id!(85, cube(&textures, [s("block/bedrock"); 6], false, None));
    id!(
        6780,
        cube(&textures, [s("block/stone_bricks"); 6], false, None)
    );
    id!(2139, cube(&textures, [s("block/bricks"); 6], false, None));
    id!(2400, cube(&textures, [s("block/obsidian"); 6], false, None));
    id!(
        2399,
        cube(&textures, [s("block/mossy_cobblestone"); 6], false, None)
    );

    id!(
        9,
        cube(
            &textures,
            [
                s("block/dirt"),
                s("block/grass_block_top"),
                s("block/grass_block_side"),
                s("block/grass_block_side"),
                s("block/grass_block_side"),
                s("block/grass_block_side"),
            ],
            false,
            Some([s("block/grass_block_side_overlay"), 1])
        )
    );
    id!(10, cube(&textures, [s("block/dirt"); 6], false, None));
    id!(
        11,
        cube(&textures, [s("block/coarse_dirt"); 6], false, None)
    );
    id!(
        12,
        cube(
            &textures,
            [
                s("block/dirt"),
                s("block/podzol_top"),
                s("block/podzol_side"),
                s("block/podzol_side"),
                s("block/podzol_side"),
                s("block/podzol_side")
            ],
            false,
            None
        )
    );
    id!(
        13,
        cube(
            &textures,
            [
                s("block/dirt"),
                s("block/podzol_top"),
                s("block/podzol_side"),
                s("block/podzol_side"),
                s("block/podzol_side"),
                s("block/podzol_side")
            ],
            false,
            None
        )
    );
    id!(118, cube(&textures, [s("block/sand"); 6], false, None));
    id!(123, cube(&textures, [s("block/red_sand"); 6], false, None));
    id!(124, cube(&textures, [s("block/gravel"); 6], false, None));
    id!(5977, cube(&textures, [s("block/clay"); 6], false, None));
    id!(
        25903,
        cube(&textures, [s("block/moss_block"); 6], false, None)
    );
    id!(
        11633,
        cube(&textures, [s("block/terracotta"); 6], false, None)
    );
    id!(
        10165,
        cube(&textures, [s("block/white_terracotta"); 6], false, None)
    );

    id!(5958, cube(&textures, [s("block/ice"); 6], false, None));
    id!(
        11635,
        cube(&textures, [s("block/packed_ice"); 6], false, None)
    );
    id!(
        13964,
        cube(&textures, [s("block/blue_ice"); 6], false, None)
    );
    id!(5959, cube(&textures, [s("block/snow"); 6], false, None));

    id!(15, cube(&textures, [s("block/oak_planks"); 6], false, None));
    id!(
        16,
        cube(&textures, [s("block/spruce_planks"); 6], false, None)
    );
    id!(
        17,
        cube(&textures, [s("block/birch_planks"); 6], false, None)
    );
    id!(
        18,
        cube(&textures, [s("block/jungle_planks"); 6], false, None)
    );
    id!(
        19,
        cube(&textures, [s("block/acacia_planks"); 6], false, None)
    );
    id!(
        20,
        cube(&textures, [s("block/cherry_planks"); 6], false, None)
    );
    id!(
        21,
        cube(&textures, [s("block/dark_oak_planks"); 6], false, None)
    );
    id!(
        25,
        cube(&textures, [s("block/pale_oak_planks"); 6], false, None)
    );
    id!(
        26,
        cube(&textures, [s("block/mangrove_planks"); 6], false, None)
    );
    id!(
        27,
        cube(&textures, [s("block/bamboo_planks"); 6], false, None)
    );
    id!(
        28,
        cube(&textures, [s("block/bamboo_mosaic"); 6], false, None)
    );

    // Logs
    id!(
        136,
        cube(
            &textures,
            [
                s("block/oak_log_top"),
                s("block/oak_log_top"),
                s("block/oak_log"),
                s("block/oak_log"),
                s("block/oak_log"),
                s("block/oak_log")
            ],
            false,
            None
        )
    );
    id!(
        137,
        cube(
            &textures,
            [
                s("block/oak_log_top"),
                s("block/oak_log_top"),
                s("block/oak_log"),
                s("block/oak_log"),
                s("block/oak_log"),
                s("block/oak_log")
            ],
            false,
            None
        )
    );
    id!(
        138,
        cube(
            &textures,
            [
                s("block/oak_log_top"),
                s("block/oak_log_top"),
                s("block/oak_log"),
                s("block/oak_log"),
                s("block/oak_log"),
                s("block/oak_log")
            ],
            false,
            None
        )
    );
    id!(
        139,
        cube(
            &textures,
            [
                s("block/spruce_log_top"),
                s("block/spruce_log_top"),
                s("block/spruce_log"),
                s("block/spruce_log"),
                s("block/spruce_log"),
                s("block/spruce_log")
            ],
            false,
            None
        )
    );
    id!(
        140,
        cube(
            &textures,
            [
                s("block/spruce_log_top"),
                s("block/spruce_log_top"),
                s("block/spruce_log"),
                s("block/spruce_log"),
                s("block/spruce_log"),
                s("block/spruce_log")
            ],
            false,
            None
        )
    );
    id!(
        141,
        cube(
            &textures,
            [
                s("block/spruce_log_top"),
                s("block/spruce_log_top"),
                s("block/spruce_log"),
                s("block/spruce_log"),
                s("block/spruce_log"),
                s("block/spruce_log")
            ],
            false,
            None
        )
    );
    id!(
        142,
        cube(
            &textures,
            [
                s("block/birch_log_top"),
                s("block/birch_log_top"),
                s("block/birch_log"),
                s("block/birch_log"),
                s("block/birch_log"),
                s("block/birch_log")
            ],
            false,
            None
        )
    );
    id!(
        143,
        cube(
            &textures,
            [
                s("block/birch_log_top"),
                s("block/birch_log_top"),
                s("block/birch_log"),
                s("block/birch_log"),
                s("block/birch_log"),
                s("block/birch_log")
            ],
            false,
            None
        )
    );
    id!(
        144,
        cube(
            &textures,
            [
                s("block/birch_log_top"),
                s("block/birch_log_top"),
                s("block/birch_log"),
                s("block/birch_log"),
                s("block/birch_log"),
                s("block/birch_log")
            ],
            false,
            None
        )
    );
    id!(
        145,
        cube(
            &textures,
            [
                s("block/jungle_log_top"),
                s("block/jungle_log_top"),
                s("block/jungle_log"),
                s("block/jungle_log"),
                s("block/jungle_log"),
                s("block/jungle_log")
            ],
            false,
            None
        )
    );
    id!(
        146,
        cube(
            &textures,
            [
                s("block/jungle_log_top"),
                s("block/jungle_log_top"),
                s("block/jungle_log"),
                s("block/jungle_log"),
                s("block/jungle_log"),
                s("block/jungle_log")
            ],
            false,
            None
        )
    );
    id!(
        147,
        cube(
            &textures,
            [
                s("block/jungle_log_top"),
                s("block/jungle_log_top"),
                s("block/jungle_log"),
                s("block/jungle_log"),
                s("block/jungle_log"),
                s("block/jungle_log")
            ],
            false,
            None
        )
    );
    id!(
        148,
        cube(
            &textures,
            [
                s("block/acacia_log_top"),
                s("block/acacia_log_top"),
                s("block/acacia_log"),
                s("block/acacia_log"),
                s("block/acacia_log"),
                s("block/acacia_log")
            ],
            false,
            None
        )
    );
    id!(
        149,
        cube(
            &textures,
            [
                s("block/acacia_log_top"),
                s("block/acacia_log_top"),
                s("block/acacia_log"),
                s("block/acacia_log"),
                s("block/acacia_log"),
                s("block/acacia_log")
            ],
            false,
            None
        )
    );
    id!(
        150,
        cube(
            &textures,
            [
                s("block/acacia_log_top"),
                s("block/acacia_log_top"),
                s("block/acacia_log"),
                s("block/acacia_log"),
                s("block/acacia_log"),
                s("block/acacia_log")
            ],
            false,
            None
        )
    );
    id!(
        151,
        cube(
            &textures,
            [
                s("block/cherry_log_top"),
                s("block/cherry_log_top"),
                s("block/cherry_log"),
                s("block/cherry_log"),
                s("block/cherry_log"),
                s("block/cherry_log")
            ],
            false,
            None
        )
    );
    id!(
        152,
        cube(
            &textures,
            [
                s("block/cherry_log_top"),
                s("block/cherry_log_top"),
                s("block/cherry_log"),
                s("block/cherry_log"),
                s("block/cherry_log"),
                s("block/cherry_log")
            ],
            false,
            None
        )
    );
    id!(
        153,
        cube(
            &textures,
            [
                s("block/cherry_log_top"),
                s("block/cherry_log_top"),
                s("block/cherry_log"),
                s("block/cherry_log"),
                s("block/cherry_log"),
                s("block/cherry_log")
            ],
            false,
            None
        )
    );
    id!(
        154,
        cube(
            &textures,
            [
                s("block/dark_oak_log_top"),
                s("block/dark_oak_log_top"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        155,
        cube(
            &textures,
            [
                s("block/dark_oak_log_top"),
                s("block/dark_oak_log_top"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        156,
        cube(
            &textures,
            [
                s("block/dark_oak_log_top"),
                s("block/dark_oak_log_top"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log"),
                s("block/dark_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        157,
        cube(
            &textures,
            [
                s("block/pale_oak_log_top"),
                s("block/pale_oak_log_top"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        158,
        cube(
            &textures,
            [
                s("block/pale_oak_log_top"),
                s("block/pale_oak_log_top"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        159,
        cube(
            &textures,
            [
                s("block/pale_oak_log_top"),
                s("block/pale_oak_log_top"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log"),
                s("block/pale_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        160,
        cube(
            &textures,
            [
                s("block/mangrove_log_top"),
                s("block/mangrove_log_top"),
                s("block/mangrove_log"),
                s("block/mangrove_log"),
                s("block/mangrove_log"),
                s("block/mangrove_log")
            ],
            false,
            None
        )
    );
    id!(
        161,
        cube(
            &textures,
            [
                s("block/mangrove_log_top"),
                s("block/mangrove_log_top"),
                s("block/mangrove_log"),
                s("block/mangrove_log"),
                s("block/mangrove_log"),
                s("block/mangrove_log")
            ],
            false,
            None
        )
    );
    id!(
        162,
        cube(
            &textures,
            [
                s("block/mangrove_log_top"),
                s("block/mangrove_log_top"),
                s("block/mangrove_log"),
                s("block/mangrove_log"),
                s("block/mangrove_log"),
                s("block/mangrove_log")
            ],
            false,
            None
        )
    );

    // Stripped logs
    id!(
        192,
        cube(
            &textures,
            [
                s("block/stripped_oak_log_top"),
                s("block/stripped_oak_log_top"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        193,
        cube(
            &textures,
            [
                s("block/stripped_oak_log_top"),
                s("block/stripped_oak_log_top"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        194,
        cube(
            &textures,
            [
                s("block/stripped_oak_log_top"),
                s("block/stripped_oak_log_top"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log"),
                s("block/stripped_oak_log")
            ],
            false,
            None
        )
    );
    id!(
        171,
        cube(
            &textures,
            [
                s("block/stripped_spruce_log_top"),
                s("block/stripped_spruce_log_top"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log")
            ],
            false,
            None
        )
    );
    id!(
        172,
        cube(
            &textures,
            [
                s("block/stripped_spruce_log_top"),
                s("block/stripped_spruce_log_top"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log")
            ],
            false,
            None
        )
    );
    id!(
        173,
        cube(
            &textures,
            [
                s("block/stripped_spruce_log_top"),
                s("block/stripped_spruce_log_top"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log"),
                s("block/stripped_spruce_log")
            ],
            false,
            None
        )
    );

    // Leaves (transparent)
    id!(278, cube(&textures, [0; 6], true, None));
    id!(306, cube(&textures, [0; 6], true, None));
    id!(334, cube(&textures, [0; 6], true, None));
    id!(362, cube(&textures, [0; 6], true, None));
    id!(390, cube(&textures, [0; 6], true, None));
    id!(418, cube(&textures, [0; 6], true, None));
    id!(446, cube(&textures, [0; 6], true, None));
    id!(502, cube(&textures, [0; 6], true, None));

    // Ores
    id!(129, cube(&textures, [s("block/gold_ore"); 6], false, None));
    id!(
        130,
        cube(
            &textures,
            [
                s("block/deepslate_top"),
                s("block/deepslate_top"),
                s("block/deepslate_gold_ore"),
                s("block/deepslate_gold_ore"),
                s("block/deepslate_gold_ore"),
                s("block/deepslate_gold_ore")
            ],
            false,
            None
        )
    );
    id!(131, cube(&textures, [s("block/iron_ore"); 6], false, None));
    id!(
        132,
        cube(
            &textures,
            [
                s("block/deepslate_top"),
                s("block/deepslate_top"),
                s("block/deepslate_iron_ore"),
                s("block/deepslate_iron_ore"),
                s("block/deepslate_iron_ore"),
                s("block/deepslate_iron_ore")
            ],
            false,
            None
        )
    );
    id!(133, cube(&textures, [s("block/coal_ore"); 6], false, None));
    id!(
        134,
        cube(
            &textures,
            [
                s("block/deepslate_top"),
                s("block/deepslate_top"),
                s("block/deepslate_coal_ore"),
                s("block/deepslate_coal_ore"),
                s("block/deepslate_coal_ore"),
                s("block/deepslate_coal_ore")
            ],
            false,
            None
        )
    );
    id!(
        135,
        cube(&textures, [s("block/nether_gold_ore"); 6], false, None)
    );
    id!(
        23970,
        cube(&textures, [s("block/copper_ore"); 6], false, None)
    );

    // Deepslate
    id!(22109, cube(&textures, [s("block/tuff"); 6], false, None));
    id!(23344, cube(&textures, [s("block/calcite"); 6], false, None));
    id!(
        25796,
        cube(&textures, [s("block/dripstone_block"); 6], false, None)
    );
    id!(
        25967,
        cube(&textures, [s("block/cobbled_deepslate"); 6], false, None)
    );

    // Nether
    id!(
        6028,
        cube(&textures, [s("block/netherrack"); 6], false, None)
    );
    id!(
        6029,
        cube(&textures, [s("block/soul_sand"); 6], false, None)
    );
    id!(13566, cube(&textures, [s("block/magma"); 6], false, None));

    // Metal blocks
    id!(
        2137,
        cube(&textures, [s("block/gold_block"); 6], false, None)
    );
    id!(
        2138,
        cube(&textures, [s("block/iron_block"); 6], false, None)
    );
    id!(
        4340,
        cube(&textures, [s("block/diamond_block"); 6], false, None)
    );
    id!(
        8449,
        cube(&textures, [s("block/emerald_block"); 6], false, None)
    );
    id!(
        565,
        cube(&textures, [s("block/lapis_block"); 6], false, None)
    );
    id!(
        11634,
        cube(&textures, [s("block/coal_block"); 6], false, None)
    );
    id!(
        20475,
        cube(&textures, [s("block/netherite_block"); 6], false, None)
    );

    // Sandstone
    id!(
        578,
        cube(
            &textures,
            [
                s("block/sandstone_bottom"),
                s("block/sandstone_top"),
                s("block/sandstone"),
                s("block/sandstone"),
                s("block/sandstone"),
                s("block/sandstone")
            ],
            false,
            None
        )
    );
    id!(
        579,
        cube(
            &textures,
            [
                s("block/sandstone_bottom"),
                s("block/sandstone_top"),
                s("block/sandstone"),
                s("block/sandstone"),
                s("block/sandstone"),
                s("block/sandstone")
            ],
            false,
            None
        )
    );
    id!(
        580,
        cube(
            &textures,
            [
                s("block/sandstone_bottom"),
                s("block/sandstone_top"),
                s("block/sandstone"),
                s("block/sandstone"),
                s("block/sandstone"),
                s("block/sandstone")
            ],
            false,
            None
        )
    );

    // Wool
    id!(
        2093,
        cube(&textures, [s("block/white_wool"); 6], false, None)
    );
    id!(
        2094,
        cube(&textures, [s("block/orange_wool"); 6], false, None)
    );
    id!(
        2095,
        cube(&textures, [s("block/magenta_wool"); 6], false, None)
    );
    id!(
        2096,
        cube(&textures, [s("block/light_blue_wool"); 6], false, None)
    );
    id!(
        2097,
        cube(&textures, [s("block/yellow_wool"); 6], false, None)
    );
    id!(
        2098,
        cube(&textures, [s("block/lime_wool"); 6], false, None)
    );
    id!(
        2099,
        cube(&textures, [s("block/pink_wool"); 6], false, None)
    );
    id!(
        2100,
        cube(&textures, [s("block/gray_wool"); 6], false, None)
    );
    id!(
        2101,
        cube(&textures, [s("block/light_gray_wool"); 6], false, None)
    );
    id!(
        2102,
        cube(&textures, [s("block/cyan_wool"); 6], false, None)
    );
    id!(
        2103,
        cube(&textures, [s("block/purple_wool"); 6], false, None)
    );
    id!(
        2104,
        cube(&textures, [s("block/blue_wool"); 6], false, None)
    );
    id!(
        2105,
        cube(&textures, [s("block/brown_wool"); 6], false, None)
    );
    id!(
        2106,
        cube(&textures, [s("block/green_wool"); 6], false, None)
    );
    id!(2107, cube(&textures, [s("block/red_wool"); 6], false, None));
    id!(
        2108,
        cube(&textures, [s("block/black_wool"); 6], false, None)
    );

    // Other
    id!(560, cube(&textures, [s("block/sponge"); 6], false, None));
    id!(562, cube(&textures, [s("block/glass"); 6], false, None));
    id!(
        2142,
        cube(
            &textures,
            [
                s("block/oak_planks"),
                s("block/oak_planks"),
                s("block/bookshelf"),
                s("block/bookshelf"),
                s("block/bookshelf"),
                s("block/bookshelf")
            ],
            false,
            None
        )
    );
    id!(
        4341,
        cube(
            &textures,
            [
                s("block/oak_planks"),
                s("block/crafting_table_top"),
                s("block/crafting_table_front"),
                s("block/crafting_table_side"),
                s("block/crafting_table_side"),
                s("block/crafting_table_front")
            ],
            false,
            None
        )
    );

    let missing_model = cube(&textures, [0; 6], false, None).into_model();

    BlockStateModelSet::new(model_by_state, missing_model, textures)
}
