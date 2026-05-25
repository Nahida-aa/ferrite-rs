// Map namings are aligned with MC Java 26.1.2 (net.minecraft.client.renderer.block.BlockAndTintGetter)

use ferrite_core::block::BlockState;
use ferrite_core::block_pos::BlockPos;

/// Trait for reading block states and light levels from the world.
/// Java: interface BlockAndTintGetter extends BlockGetter, LevelHeightAccessor
pub trait BlockAndTintGetter {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn get_light_emission(&self, pos: BlockPos) -> i32;
    fn is_block_air(&self, pos: BlockPos) -> bool {
        self.get_block_state(pos).is_air()
    }
}
