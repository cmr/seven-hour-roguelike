use {ecs, generation};
use self::MessageKind::*;
use entities::Loc;

use std::collections::{HashMap, VecDeque};
use rustbox::{self, Key, Event, Color};

pub struct Game {
    ecs: ecs::ComponentStorage,
    player: ecs::Entity,
    floors: Vec<generation::Floor>,
    current_floor: isize,
    ent_types: HashMap<&'static str, ecs::EntityType>,
    components: HashMap<&'static str, ecs::Component>,
    term: ::rustbox::RustBox,
    messages: VecDeque<(String, MessageKind)>,
    scroll_frame: (i16, i16),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum MessageKind {
    Temp,
    Critical,
    Notice
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Dir {
    Up, Down, Left, Right, UpLeft, UpRight, DownLeft, DownRight
}

impl MessageKind {
    fn style(&self) -> rustbox::Style { rustbox::RB_NORMAL }

    fn fg(&self) -> Color {
        match *self {
            Temp => Color::Default,
            Critical => Color::Red,
            Notice => Color::Blue,
        }
    }

    fn bg(&self) -> Color { Color::Default }
}

impl Game {
    pub fn new() -> Game {
        let mut ecs = ecs::ComponentStorage::new();

        let gold = ecs.create_component(2);
        let hp = ecs.create_component(2);
        let ai = ecs.create_component(2);
        let location = ecs.create_component(4);
        let spells = ecs.create_component(4);
        let inventory = ecs.create_component(4);
        let equipment = ecs.create_component(4);

        let item_id = ecs.create_component(4);
        let item_count = ecs.create_component(2);

        let monster_type = ecs.create_entity_type(vec![gold, hp, ai, location, spells, inventory, equipment]);
        let item_type = ecs.create_entity_type(vec![item_id, item_count, location]);

        let player = ecs.create_entity(monster_type);

        let mut comps = HashMap::new();

        comps.insert("gold", gold);
        comps.insert("hp", hp);
        comps.insert("ai", ai);
        comps.insert("location", location);
        comps.insert("spells", spells);
        comps.insert("inventory", inventory);
        comps.insert("equipment", equipment);
        comps.insert("item_id", item_id);
        comps.insert("item_count", item_count);

        let mut itys = HashMap::new();

        itys.insert("item", item_type);
        itys.insert("monster", monster_type);

        Game {
            ecs: ecs,
            player: player,
            ent_types: itys,
            floors: generation::dungeon(),
            current_floor: 0,
            components: comps,
            term: ::rustbox::RustBox::init(::std::default::Default::default()).unwrap(),
            messages: VecDeque::new(),
            scroll_frame: (0, 1),
        }
    }

    pub fn play(&mut self) {
        let loc_ptr = self.ecs.lookup_component(self.player, *self.components.get("location").unwrap()).unwrap();
        let data_ptr = unsafe { ::std::mem::transmute::<*mut u8, &mut [i16; 2]>(loc_ptr) };

        *data_ptr = [15, 15];

        let frame_delay = ::std::time::duration::Duration::milliseconds(1000);

        self.post_message(Notice, "Hello!".to_string());
        self.redraw(None);
        loop {
            let evt = self.term.peek_event(frame_delay, false).unwrap();
            match evt {
                Event::KeyEvent(Some(Key::Char('q'))) => return,
                Event::KeyEvent(Some(k)) => self.process_key(k),
                Event::ResizeEvent(x, y) => self.redraw(Some((x, y))),
                _ => { }
            }
        }
    }

    fn redraw(&mut self, new_size: Option<(i32, i32)>) {
        self.term.clear();
        if self.draw_message() && new_size.is_none() {
            self.messages.pop_front();
        }

        self.draw_floor();
        self.draw_entities();
        self.draw_player();

        self.term.present();
    }

    /// Draw status message, returning true if a message wants to be popped.
    fn draw_message(&mut self) -> bool {
        let mut pop = false;
        match self.messages.front() {
            Some(&(ref n, p)) => {
                self.term.print(0, 0, p.style(), p.fg(), p.bg(), &n);
                if p == Temp { pop = true; }
            },
            None => self.term.print(0, 0, rustbox::RB_NORMAL, Color::Default, Color::Default, "... nothing yet ...")
        }

        pop
    }

    fn process_key(&mut self, k: Key) {
        match k {
            Key::Char(']') => { self.messages.pop_front(); },
            Key::Up => self.move_player(Dir::Up, 1),
            Key::Down => self.move_player(Dir::Down, 1),
            Key::Left => self.move_player(Dir::Left, 1),
            Key::Right => self.move_player(Dir::Right, 1),
            Key::Char('i') => { self.move_player(Dir::UpLeft, 1); },
            Key::Char('o') => { self.move_player(Dir::UpRight, 1); },
            Key::Char('k') => { self.move_player(Dir::DownLeft, 1); },
            Key::Char('l') => { self.move_player(Dir::DownRight, 1); },
            k => self.post_message(Temp, format!("I don't know how to {:?}", k)),
        }

        self.redraw(None)
    }

    fn post_message(&mut self, k: MessageKind, s: String) {
        if k == Temp && self.messages.front().map_or(false, |&(_, k)| k == Temp) {
            self.messages.pop_front();
            self.messages.push_front((s, k))
        } else if k == Temp {
            self.messages.push_front((s, k));
        } else {
            self.messages.push_back((s, k))
        }
    }

    fn draw_floor(&mut self) {
        let mut to_draw = Vec::new();
        {
        let fl = &self.floors[self.current_floor as usize];

        for room in &fl.rooms {
            let orig = room.0;
            for x in 0 .. room.1 {
                for y in 0 .. room.2 {
                    if x == 0 || y == 0 || x == room.1 - 1 || y == room.2 - 1 {
                        // wall
                        to_draw.push(([orig[0] + x, orig[1] + y], '='));
                    } else {
                        // ground
                        to_draw.push(([orig[0] + x, orig[1] + y], '.'))
                    }
                }
            }
        }

        for line in &fl.lines {
            ::util::bresenham(line.0, line.1, |x, y| { to_draw.push(([x, y], '.')); false });
        }
        }

        for (loc, c) in to_draw {
            self.draw_trans_char(loc, c, rustbox::RB_NORMAL, Color::Default, Color::Default);
        }

    }

    fn draw_entities(&mut self) {

    }

    fn draw_trans_char(&mut self, mut pos: Loc, c: char, s: rustbox::Style, fg: Color, bg: Color) {
        pos[0] += self.scroll_frame.0;
        pos[1] += self.scroll_frame.1;
        if pos[0] < self.scroll_frame.0 || pos[1] < self.scroll_frame.1 { return; }
        self.term.print_char(pos[0] as usize, pos[1] as usize, s, fg, bg, c);
    }

    fn draw_player(&mut self) {
        let loc_ptr = self.ecs.lookup_component(self.player, *self.components.get("location").unwrap()).unwrap();
        let data = unsafe { *::std::mem::transmute::<*mut u8, &mut [i16; 2]>(loc_ptr) };
        self.draw_trans_char(data, '@', rustbox::RB_NORMAL, Color::Green, Color::Default);
    }

    fn move_player(&mut self, dir: Dir, mag: i16) {
        let loc_ptr = self.ecs.lookup_component(self.player, *self.components.get("location").unwrap()).unwrap();
        let data_ptr = unsafe { ::std::mem::transmute::<*mut u8, &mut [i16; 2]>(loc_ptr) };

        let mut data = *data_ptr;

        match dir {
            Dir::Down => { data[1] += mag; }
            Dir::Up => { data[1] -= mag; }
            Dir::Left => { data[0] -= mag; }
            Dir::Right => { data[0] += mag; }
            Dir::DownLeft => { data[0] -= mag; data[1] += mag; }
            Dir::DownRight => { data[0] += mag; data[1] += mag; }
            Dir::UpLeft => { data[0] -= mag; data[1] -= mag; }
            Dir::UpRight => { data[0] += mag; data[1] -= mag; }
        }

        let pl = self.player;
        *data_ptr = self.process_move(pl, data, true)
    }

    /// Try to move to a new position, processing any triggers, and returning the position the
    /// character actually ended up at.
    fn process_move(&mut self, ent: ecs::Entity, new_pos: Loc, normal: bool) -> Loc {
        let loc_ptr = self.ecs.lookup_component(ent, *self.components.get("location").unwrap()).unwrap();
        let old_pos = unsafe { *::std::mem::transmute::<*mut u8, &mut [i16; 2]>(loc_ptr) };

        let fl = &self.floors[self.current_floor as usize];

        let mut valid = false;
        for room in &fl.rooms {
            if new_pos[0] > room.0[0] && new_pos[1] > room.0[1]
                    && new_pos[0] < room.0[0] + room.1
                    && new_pos[1] < room.0[1] + room.2 {
                valid = true;
                break;
            }
        }
        if !valid {
            for line in &fl.lines {
                ::util::bresenham(line.0, line.1, |x, y| if new_pos[0] == x && new_pos[1] == y { valid = true; true } else { false });
            }
        }

        if !valid {
            old_pos
        } else {
            new_pos
        }
    }
}
