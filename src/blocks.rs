use enum_map::Enum;

#[derive(Debug, Clone, Copy, Enum)]
pub enum Direction {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

#[derive(Debug, Clone, Copy)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
}

impl Block {
    pub fn raycast_solid(&self) -> bool {
        match *self {
            Block::Air => false,
            Block::Grass | Block::Dirt | Block::Stone => true,
        }
    }

    pub fn soild_in_direction(&self, #[expect(unused)] direction: Direction) -> bool {
        match *self {
            Block::Air => false,
            Block::Grass | Block::Dirt | Block::Stone => true,
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        match *self {
            Block::Air => (1.0, 1.0, 1.0),
            Block::Grass => (0.2, 0.6, 0.3),
            Block::Dirt => (0.5, 0.4, 0.2),
            Block::Stone => (0.5, 0.5, 0.5),
        }
    }
}
