// Map namings are aligned with MC Java 26.1.2 (net.minecraft.core.Vec3i)

use crate::direction::{Axis, Direction};

/// Trait mirroring Java's Vec3i base class methods.
/// BlockPos and SectionPos both implement this.
pub trait Vec3i: Clone + Copy + PartialEq + Eq {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn z(&self) -> i32;
    fn with_x(&self, x: i32) -> Self;
    fn with_y(&self, y: i32) -> Self;
    fn with_z(&self, z: i32) -> Self;

    fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        if dx == 0 && dy == 0 && dz == 0 {
            return *self;
        }
        self.with_x(self.x() + dx).with_y(self.y() + dy).with_z(self.z() + dz)
    }

    fn relative(&self, direction: Direction, steps: i32) -> Self {
        if steps == 0 {
            return *self;
        }
        self.offset(direction.step_x() * steps, direction.step_y() * steps, direction.step_z() * steps)
    }

    fn relative_axis(&self, axis: Axis, steps: i32) -> Self {
        if steps == 0 {
            return *self;
        }
        let x = if axis == Axis::X { steps } else { 0 };
        let y = if axis == Axis::Y { steps } else { 0 };
        let z = if axis == Axis::Z { steps } else { 0 };
        self.offset(x, y, z)
    }

    fn above(&self) -> Self {
        self.above_by(1)
    }

    fn above_by(&self, steps: i32) -> Self {
        self.relative(Direction::Up, steps)
    }

    fn below(&self) -> Self {
        self.below_by(1)
    }

    fn below_by(&self, steps: i32) -> Self {
        self.relative(Direction::Down, steps)
    }

    fn north(&self) -> Self {
        self.north_by(1)
    }

    fn north_by(&self, steps: i32) -> Self {
        self.relative(Direction::North, steps)
    }

    fn south(&self) -> Self {
        self.south_by(1)
    }

    fn south_by(&self, steps: i32) -> Self {
        self.relative(Direction::South, steps)
    }

    fn west(&self) -> Self {
        self.west_by(1)
    }

    fn west_by(&self, steps: i32) -> Self {
        self.relative(Direction::West, steps)
    }

    fn east(&self) -> Self {
        self.east_by(1)
    }

    fn east_by(&self, steps: i32) -> Self {
        self.relative(Direction::East, steps)
    }

    fn closer_than(&self, pos: &impl Vec3i, distance: f64) -> bool {
        self.dist_sqr(pos) < distance * distance
    }

    fn dist_sqr(&self, pos: &impl Vec3i) -> f64 {
        let dx = self.x() as f64 - pos.x() as f64;
        let dy = self.y() as f64 - pos.y() as f64;
        let dz = self.z() as f64 - pos.z() as f64;
        dx * dx + dy * dy + dz * dz
    }

    fn dist_manhattan(&self, pos: &impl Vec3i) -> i32 {
        (pos.x() - self.x()).abs() + (pos.y() - self.y()).abs() + (pos.z() - self.z()).abs()
    }

    fn dist_chessboard(&self, pos: &impl Vec3i) -> i32 {
        ((self.x() - pos.x()).abs())
            .max((self.y() - pos.y()).abs())
            .max((self.z() - pos.z()).abs())
    }

    fn get_axis(&self, axis: Axis) -> i32 {
        axis.choose(self.x(), self.y(), self.z())
    }
}
