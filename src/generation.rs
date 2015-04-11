use ecs;
use rand;
use entities::Loc;

pub struct Floor {
    pub rooms: Vec<(Loc, i16, i16)>,
    pub lines: Vec<(Loc, Loc)>,
    pub monsters: Vec<ecs::Entity>,
    pub stairs: Vec<ecs::Entity>,
}

pub fn dungeon() -> Vec<Floor> {
    let mut floors = vec![];


    floors.push(Floor {
        rooms: vec![([10, 10], 20, 20)],
        lines: vec![([5,5], [25, 25])],
        monsters: vec![],
        stairs: vec![],
    });

    floors
}

