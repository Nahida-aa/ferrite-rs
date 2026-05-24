use glam::DVec3;

#[derive(Clone, Debug)]
pub struct EntityPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for EntityPosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct EntityLook {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for EntityLook {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.3 }
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub position: EntityPosition,
    pub look: EntityLook,
    pub velocity: DVec3,
    pub on_ground: bool,
}
