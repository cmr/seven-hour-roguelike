pub type Loc = [i16; 2];

#[repr(C, packed)]
struct MonsterEntityType {
    gold: u16,
    hp: i16,
    ai: u16,
    location: Loc,
    spells: Option<u16>,
    inventory: Option<u16>,
    equipment: Option<u16>,
}

#[repr(C, packed)]
struct ItemEntityType {
    id: u32,
    count: u16,
    location: Loc,
}
