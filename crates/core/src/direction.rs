// Map namings are aligned with MC Java 26.1.2 (net.minecraft.core.Direction)

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

pub const DOWN: usize = 0;
pub const UP: usize = 1;
pub const NORTH: usize = 2;
pub const SOUTH: usize = 3;
pub const WEST: usize = 4;
pub const EAST: usize = 5;

pub const ALL_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

impl Direction {
    pub fn step_x(&self) -> i32 {
        match self {
            Direction::East => 1,
            Direction::West => -1,
            _ => 0,
        }
    }

    pub fn step_y(&self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
            _ => 0,
        }
    }

    pub fn step_z(&self) -> i32 {
        match self {
            Direction::South => 1,
            Direction::North => -1,
            _ => 0,
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    pub fn axis(&self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }

    pub fn axis_direction(&self) -> AxisDirection {
        match self {
            Direction::Down | Direction::North | Direction::West => AxisDirection::Negative,
            Direction::Up | Direction::South | Direction::East => AxisDirection::Positive,
        }
    }

    pub fn is_vertical(&self) -> bool {
        self.axis().is_vertical()
    }

    pub fn is_horizontal(&self) -> bool {
        self.axis().is_horizontal()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const ALL: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    pub fn is_vertical(&self) -> bool {
        matches!(self, Axis::Y)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Axis::X | Axis::Z)
    }

    pub fn choose(&self, x: i32, y: i32, z: i32) -> i32 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    pub fn choose_f64(&self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    pub fn positive_direction(&self) -> Direction {
        match self {
            Axis::X => Direction::East,
            Axis::Y => Direction::Up,
            Axis::Z => Direction::South,
        }
    }

    pub fn negative_direction(&self) -> Direction {
        match self {
            Axis::X => Direction::West,
            Axis::Y => Direction::Down,
            Axis::Z => Direction::North,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AxisDirection {
    Positive,
    Negative,
}

impl AxisDirection {
    pub fn step(&self) -> i32 {
        match self {
            AxisDirection::Positive => 1,
            AxisDirection::Negative => -1,
        }
    }

    pub fn opposite(&self) -> AxisDirection {
        match self {
            AxisDirection::Positive => AxisDirection::Negative,
            AxisDirection::Negative => AxisDirection::Positive,
        }
    }
}
