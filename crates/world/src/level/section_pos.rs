// Map namings are aligned with MC Java 26.1.2 (net.minecraft.core.SectionPos)

use ferrite_core::direction::Direction;
use ferrite_core::block_pos::BlockPos;
use ferrite_core::vec3i::Vec3i;
use crate::level::ChunkPos;

pub const SECTION_BITS: i32 = 4;
pub const SECTION_SIZE: i32 = 16;
pub const SECTION_MASK: i32 = 15;
pub const SECTION_HALF_SIZE: i32 = 8;
pub const SECTION_MAX_INDEX: i32 = 15;

const PACKED_Y_LENGTH: i32 = 20;
const PACKED_Z_LENGTH: i32 = 22;
const PACKED_X_MASK: i64 = 0x3FFFFF;
const PACKED_Y_MASK: i64 = 0xFFFFF;
const PACKED_Z_MASK: i64 = 0x3FFFFF;
const Y_OFFSET: i32 = 0;
const Z_OFFSET: i32 = 20;
const X_OFFSET: i32 = 42;
const RELATIVE_X_SHIFT: i32 = 8;
const RELATIVE_Y_SHIFT: i32 = 0;
const RELATIVE_Z_SHIFT: i32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SectionPos(i32, i32, i32);

impl SectionPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(x, y, z)
    }

    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self(
            Self::block_to_section_coord(pos.x()),
            Self::block_to_section_coord(pos.y()),
            Self::block_to_section_coord(pos.z()),
        )
    }

    pub fn from_chunk_pos(pos: ChunkPos, section_y: i32) -> Self {
        Self(pos.0, section_y, pos.1)
    }

    pub fn from_packed(packed: i64) -> Self {
        Self(Self::x(packed), Self::y(packed), Self::z(packed))
    }

    // --- Packed long encoding ---

    pub fn as_long_from_block_pos(pos: BlockPos) -> i64 {
        Self::as_long_xyz(
            Self::block_to_section_coord(pos.x()),
            Self::block_to_section_coord(pos.y()),
            Self::block_to_section_coord(pos.z()),
        )
    }

    pub fn as_long(&self) -> i64 {
        Self::as_long_xyz(self.x(), self.y(), self.z())
    }

    pub fn as_long_xyz(x: i32, y: i32, z: i32) -> i64 {
        ((x as i64 & PACKED_X_MASK) << X_OFFSET as u64)
            | ((y as i64 & PACKED_Y_MASK) << Y_OFFSET as u64)
            | ((z as i64 & PACKED_Z_MASK) << Z_OFFSET as u64)
    }

    pub fn offset_packed(packed: i64, direction: Direction) -> i64 {
        Self::offset_packed_by(
            packed,
            direction.step_x(),
            direction.step_y(),
            direction.step_z(),
        )
    }

    pub fn offset_packed_by(packed: i64, step_x: i32, step_y: i32, step_z: i32) -> i64 {
        Self::as_long_xyz(
            Self::x(packed) + step_x,
            Self::y(packed) + step_y,
            Self::z(packed) + step_z,
        )
    }

    // --- Block ↔ Section conversion ---

    pub fn pos_to_section_coord(pos: f64) -> i32 {
        Self::block_to_section_coord_f64(pos)
    }

    pub fn block_to_section_coord(block_coord: i32) -> i32 {
        block_coord >> SECTION_BITS
    }

    pub fn block_to_section_coord_f64(coord: f64) -> i32 {
        (coord.floor() as i32) >> SECTION_BITS
    }

    pub fn section_relative(block_coord: i32) -> i32 {
        block_coord & SECTION_MASK
    }

    pub fn section_relative_pos(pos: BlockPos) -> u16 {
        let rx = Self::section_relative(pos.x()) as u16;
        let ry = Self::section_relative(pos.y()) as u16;
        let rz = Self::section_relative(pos.z()) as u16;
        rx << RELATIVE_X_SHIFT | rz << RELATIVE_Z_SHIFT | ry << RELATIVE_Y_SHIFT
    }

    pub fn section_relative_x(relative: u16) -> i32 {
        (relative >> RELATIVE_X_SHIFT) as i32 & SECTION_MASK
    }

    pub fn section_relative_y(relative: u16) -> i32 {
        (relative >> RELATIVE_Y_SHIFT) as i32 & SECTION_MASK
    }

    pub fn section_relative_z(relative: u16) -> i32 {
        (relative >> RELATIVE_Z_SHIFT) as i32 & SECTION_MASK
    }

    pub fn section_to_block_coord(section_coord: i32) -> i32 {
        section_coord << SECTION_BITS
    }

    pub fn section_to_block_coord_with_offset(section_coord: i32, offset: i32) -> i32 {
        Self::section_to_block_coord(section_coord) + offset
    }

    // --- Packed long field extraction ---

    pub fn x(packed: i64) -> i32 {
        ((packed << 0) >> X_OFFSET) as i32
    }

    pub fn y(packed: i64) -> i32 {
        // sign-extend the lower 20 bits
        ((packed << (64 - PACKED_Y_LENGTH)) >> (64 - PACKED_Y_LENGTH)) as i32
    }

    pub fn z(packed: i64) -> i32 {
        ((packed << (64 - X_OFFSET - PACKED_Z_LENGTH)) >> X_OFFSET) as i32
    }

    // --- Instance methods ---

    pub fn min_block_x(&self) -> i32 {
        Self::section_to_block_coord(self.x())
    }

    pub fn min_block_y(&self) -> i32 {
        Self::section_to_block_coord(self.y())
    }

    pub fn min_block_z(&self) -> i32 {
        Self::section_to_block_coord(self.z())
    }

    pub fn max_block_x(&self) -> i32 {
        Self::section_to_block_coord_with_offset(self.x(), SECTION_MAX_INDEX)
    }

    pub fn max_block_y(&self) -> i32 {
        Self::section_to_block_coord_with_offset(self.y(), SECTION_MAX_INDEX)
    }

    pub fn max_block_z(&self) -> i32 {
        Self::section_to_block_coord_with_offset(self.z(), SECTION_MAX_INDEX)
    }

    pub fn origin(&self) -> BlockPos {
        BlockPos(
            Self::section_to_block_coord(self.x()),
            Self::section_to_block_coord(self.y()),
            Self::section_to_block_coord(self.z()),
        )
    }

    pub fn center(&self) -> BlockPos {
        self.origin()
            .offset(SECTION_HALF_SIZE, SECTION_HALF_SIZE, SECTION_HALF_SIZE)
    }

    pub fn chunk(&self) -> ChunkPos {
        ChunkPos(self.x(), self.z())
    }

    pub fn relative_to_block_x(&self, relative: u16) -> i32 {
        self.min_block_x() + Self::section_relative_x(relative)
    }

    pub fn relative_to_block_y(&self, relative: u16) -> i32 {
        self.min_block_y() + Self::section_relative_y(relative)
    }

    pub fn relative_to_block_z(&self, relative: u16) -> i32 {
        self.min_block_z() + Self::section_relative_z(relative)
    }

    pub fn relative_to_block_pos(&self, relative: u16) -> BlockPos {
        BlockPos(
            self.relative_to_block_x(relative),
            self.relative_to_block_y(relative),
            self.relative_to_block_z(relative),
        )
    }

    pub fn block_to_section(block_node: i64) -> i64 {
        Self::as_long_xyz(
            Self::block_to_section_coord(BlockPos::x_from_packed(block_node)),
            Self::block_to_section_coord(BlockPos::y_from_packed(block_node)),
            Self::block_to_section_coord(BlockPos::z_from_packed(block_node)),
        )
    }

    pub fn get_zero_node(x: i32, z: i32) -> i64 {
        Self::as_long_xyz(x, 0, z) & (0xFFFF_FFFF_FFF0_0000_u64 as i64)
    }

    pub fn section_to_chunk(packed: i64) -> i64 {
        ChunkPos::pack(Self::x(packed), Self::z(packed))
    }
}

impl Vec3i for SectionPos {
    fn x(&self) -> i32 {
        self.0
    }
    fn y(&self) -> i32 {
        self.1
    }
    fn z(&self) -> i32 {
        self.2
    }
    fn with_x(&self, x: i32) -> Self {
        Self(x, self.1, self.2)
    }
    fn with_y(&self, y: i32) -> Self {
        Self(self.0, y, self.2)
    }
    fn with_z(&self, z: i32) -> Self {
        Self(self.0, self.1, z)
    }
}
