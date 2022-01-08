use std::cmp;

use tcod::colors::*;
use tcod::console::*;
use tcod::chars;
use tcod::input::Key;
use tcod::input::KeyCode::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
const COLOR_DARK_WALL: Color = Color {r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: Color = Color {r: 50, g: 50, b: 150};

struct Tcod {
    root: Root,
    con: Offscreen,
}

struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Self {x, y, char, color}
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, map: &Map) {
        let x = self.x + dx;
        let y = self.y + dy;
        if !map[x as usize][y as usize].blocked {
            self.x = x;
            self.y = y;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Self {blocked: false, block_sight: false}
    }

    pub fn wall() -> Self {
        Self {blocked: true, block_sight: true}
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32, x2: i32, y1: i32, y2: i32,
}

impl Rect {
    pub fn new(x1: i32, y1: i32, w: i32, h: i32) -> Self {
        Self { x1, x2: x1 + w, y1, y2: y1 + h}
    }
}

fn add_room(room: Rect, map: &mut Map) {
    for x in room.x1 + 1..room.x2 {
        for y in room.y1 + 1..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn add_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    (cmp::min(x1, x2)..cmp::max(x1, x2) + 1).for_each(|x| 
        map[x as usize][y as usize] = Tile::empty() );
}

fn add_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    (cmp::min(y1, y2)..cmp::max(y1, y2) + 1).for_each(|y| 
        map[x as usize][y as usize] = Tile::empty() );
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
    objects: Vec<Object>,
}

fn make_map() -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(50, 15, 10, 15);
    add_room(room1, &mut map);
    add_room(room2, &mut map);
    add_h_tunnel(25, 55, 23, &mut map);
    map
}

fn render_all(tcod: &mut Tcod, game: &Game) {
    // draw all objects
    for object in &game.objects {
        object.draw(&mut tcod.con);
    }

    // draw other stuff
    (0..MAP_HEIGHT).for_each(|y| (0..MAP_WIDTH).for_each(|x| 
        if game.map[x as usize][y as usize].block_sight {
            tcod.con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
        } else {
            tcod.con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
        }
    ));

    // add offscreen to screen
    blit(&tcod.con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), &mut tcod.root, (0, 0), 1.0, 1.0);
}

fn game_loop(mut tcod: Tcod, mut game: Game) {
    while !tcod.root.window_closed() {
        
        // render stuff
        tcod.con.clear();

        render_all(&mut tcod, &game);
        
        tcod.root.flush();

        if handle_keys(&mut tcod, &mut game.objects[0], &game.map) {
            break;
        }
        //tcod.root.wait_for_keypress(true);
    }
}

fn handle_keys(tcod: &mut Tcod, player: &mut Object, map: &Map) -> bool {
    let Key {code, alt, ..} = tcod.root.wait_for_keypress(true);
    match (code, alt) {
        (Up, _)     => player.move_by(0, -1, &map),
        (Down, _)   => player.move_by(0, 1, &map),
        (Left, _)   => player.move_by(-1, 0, &map),
        (Right, _)  => player.move_by(1, 0, &map),
        (Escape, _) => return true,
        _ => (),
    }
    false
}

fn main() {
    let root = Root::initializer()
        .font("resources/Talryth_square_15x15.png", FontLayout::AsciiInRow)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust Roguelike")
        .init();
    tcod::system::set_fps(LIMIT_FPS);
    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let tcod = Tcod { root, con };
    let player = Object::new(25, 23, '@', WHITE);
    let npc = Object::new(25, 25, '@', YELLOW);
    let objects = vec![player, npc];

    let game = Game {map: make_map(), objects};

    game_loop(tcod, game);
}
