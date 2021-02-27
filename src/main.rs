mod keys;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::vec_deque::Drain;
use std::env;
use std::time::Instant;
use std::convert::TryFrom;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use neovim_lib::{Neovim, NeovimApi, Session, UiAttachOptions, Value};

type AtlasIndexKey = u64;
type NvimRow = usize;
type NvimColumn = usize;
type NvimWidth = usize;
type NvimHeight = usize;
type NvimGridId = u64;

enum Damage {
    Cell {
        row: NvimRow,
        column: NvimColumn,
        width: NvimWidth,
        height: NvimHeight,
    },
    VerticalScroll {
        to: NvimRow,
        from: NvimRow,
        height: NvimHeight,
    }
}

pub struct NvimGrid {
    chars: Vec<Vec<Option<char>>>,
    colors: Vec<Vec<u64>>,
    cursor: (NvimRow, NvimColumn),
    damages: Vec<Damage>, 
}

impl NvimGrid {
    pub fn new (width: NvimWidth, height: NvimHeight) -> NvimGrid {
        NvimGrid {
            chars: vec![vec![Some(' '); width]; height],
            colors: vec![vec![0; width]; height],
            cursor: (0, 0),
            damages: vec![],
        }
    }
    pub fn get_height(&self) -> NvimHeight {
        assert!(self.chars.len() == self.colors.len());
        self.chars.len() as NvimHeight
    }
    pub fn get_width(&self) -> NvimWidth {
        (if self.chars.len() < 1 {
            0
        } else {
            assert!(self.chars[0].len() == self.colors[0].len());
            self.chars[0].len()
        }) as NvimWidth
    }
    pub fn get_cursor_pos(&self) -> (NvimRow, NvimColumn) {
        match (self.cursor.0 < self.get_height(), self.cursor.1 < self.get_width()) {
            (true , true)  => (self.cursor.0, self.cursor.1),
            (true , false) => (self.cursor.0, self.get_width() - 1),
            (false, true)  => (self.get_height() - 1, self.cursor.1),
            _ => (self.get_width() - 1, self.get_height() - 1),
        }
    }
    pub fn set_cursor_pos(&mut self, row: NvimRow, column: NvimColumn) {
        self.cursor.0 = row;
        self.cursor.1 = column;
    }
}

fn to_sdl_color (color: u64) -> Color {
    Color::RGB(
        ((color & 0x00ff_0000) >> 16) as u8,
        ((color & 0x0000_ff00) >> 8) as u8,
         (color & 0x0000_00ff) as u8
    )
}

pub struct NvimHighlightAttribute {
    background: Option<Color>,
    foreground: Option<Color>,
    special: Option<Color>,
    blend: u8,
    bold: bool,
    italic: bool,
    reverse: bool,
    strikethrough: bool,
    undercurl: bool,
    underline: bool,
}

impl NvimHighlightAttribute {
    pub fn new () -> NvimHighlightAttribute {
        NvimHighlightAttribute {
            background: None,
            foreground: None,
            special: None,
            blend: 0,
            bold: false,
            italic: false,
            reverse: false,
            strikethrough: false,
            undercurl: false,
            underline: false,
        }
    }
}

pub struct NvimState {
    grids: HashMap<NvimGridId, NvimGrid>,
    hl_attrs: HashMap<u64, NvimHighlightAttribute>,
}

impl NvimState {
    pub fn new () -> NvimState {
        NvimState {
            grids: HashMap::new(),
            hl_attrs: HashMap::new(),
        }
    }
    pub fn default_colors_set (&mut self, rgb_fg: Option<u64>, rgb_bg: Option<u64>, rgb_sp: Option<u64>) {
        let id = 0;
        let high = if let Some (a) = self.hl_attrs.get_mut(&id) {
            a
        } else {
            self.hl_attrs.insert(id, NvimHighlightAttribute::new());
            self.hl_attrs.get_mut(&id).unwrap()
        };
        high.foreground = rgb_fg.map(|c| to_sdl_color(c));
        high.background = rgb_bg.map(|c| to_sdl_color(c));
        high.special = rgb_sp.map(|c| to_sdl_color(c));
        for (_, g) in self.grids.iter_mut() {
            g.damages.push(Damage::Cell { row: 0, column: 0, width: g.get_width(), height: g.get_height() });
        }
    }
    pub fn grid_clear (&mut self, id: NvimGridId) {
        let grid = self.grids.get_mut(&id).unwrap();
        for row in 0 .. grid.get_height() {
            for column in 0 .. grid.get_width() {
                grid.chars[row][column] = None;
                grid.colors[row][column] = 0;
            }
        }
    }
    pub fn grid_cursor_goto (&mut self, id: NvimGridId, row: NvimRow, column: NvimColumn) {
        let grid = self.grids.get_mut(&id).unwrap();
        let old_pos = grid.get_cursor_pos();
        grid.set_cursor_pos(row, column);
        grid.damages.push(Damage::Cell { row: old_pos.0, column: old_pos.1, width: 1, height: 1 });
    }
    pub fn grid_resize (&mut self, id: NvimGridId, width: NvimWidth, height: NvimHeight) {
        let grid = if let Some(g) = self.grids.get_mut(&id) {
            g
        } else {
            self.grids.insert(id, NvimGrid::new(0, 0));
            self.grids.get_mut(&id).unwrap()
        };
        if grid.get_height() > height {
            grid.chars.truncate(height);
            grid.colors.truncate(height);
        } else {
            grid.damages.push(Damage::Cell { row:                     grid.get_height(), column:                    0, width:                    width, height:                    height - grid.get_height() }); 
            for _count in grid.get_height() .. height {
                grid.chars.push(vec![None; width]);
                grid.colors.push(vec![0; width]);
            }
        }
        if grid.get_width() != width {
            if grid.get_width() < width {
                grid.damages.push(Damage::Cell { row:                         0, column:                        grid.get_width(), width:                        width - grid.get_width(), height:                        grid.get_height() }); 
            }
            for row in 0 .. grid.get_height() {
                grid.chars[row].resize(width as usize, Some(' '));
                grid.colors[row].resize(width as usize, 0);
            }
        }
    }
    pub fn grid_line (&mut self, id: NvimGridId, row: NvimRow, col_start: NvimColumn, cells: &Vec<Value>) {
        let grid = self.grids.get_mut(&id).unwrap();
        let chars = &mut grid.chars[row as usize];
        let colors = &mut grid.colors[row as usize];
        let mut prev_column = col_start as usize;
        let mut prev_color = 0;
        let mut damage_length: NvimWidth = 0;
        for cell in cells {
            let mut c = cell.as_array().unwrap().into_iter();
            let char = c.next().unwrap();
            if let Some(Value::Integer(color)) = c.next() {
                prev_color = color.as_u64().unwrap();
            }
            let repeat = (if let Some(Value::Integer(r)) = c.next() {
                r.as_u64().unwrap()
            } else {
                1
            }) as NvimWidth;
            for _times in 0 .. repeat {
                chars[prev_column] = char.as_str().unwrap().chars().next();
                colors[prev_column] = prev_color;
                prev_column += 1;
            }
            damage_length += repeat;
        }
        grid.damages.push(Damage::Cell { row: row, column: col_start, width: damage_length, height: 1 });
    }
    pub fn grid_scroll (&mut self, id: NvimGridId, top: NvimRow, bot: NvimRow, _left: NvimColumn, _right: NvimColumn, rows: i64, _cols: i64) {
        assert!(_cols == 0);
        let grid = self.grids.get_mut(&id).unwrap();
        if rows > 0 {
            // Moving characters up
            let r : usize = rows as usize;
            let bottom = if (bot + r) >= grid.get_height() {
                grid.get_height() - r
            } else {
                bot
            };
            for y in top .. bottom {
                for x in 0 .. grid.get_width() {
                    grid.chars[y][x] = grid.chars[y + r][x];
                    grid.colors[y][x] = grid.colors[y + r][x];
                }
            }
            grid.damages.push(Damage::VerticalScroll { from: top + r, to: top, height: bottom - top });
        } else if rows < 0 {
            // Moving characters down
            let mut y = bot - 1;
            while y >= top && ((y as i64) + rows) >= 0 {
                for x in 0 .. grid.get_width() {
                    grid.chars[y][x] = grid.chars[((y as i64) + rows) as usize][x];
                    grid.colors[y][x] = grid.colors[((y as i64) + rows) as usize][x];
                }
                y -= 1
            }
            // You don't have to understand this, just know it works.
            grid.damages.push(Damage::VerticalScroll {
                from: top,
                to: top + (rows.abs() as usize),
                height: bot - 1 - y,
            });
        }
    }
    pub fn hl_attr_define (&mut self, id: u64, map: &Vec<(Value, Value)>) {
        let attr = if let Some(a) = self.hl_attrs.get_mut(&id) {
            a
        } else {
            self.hl_attrs.insert(id, NvimHighlightAttribute::new());
            self.hl_attrs.get_mut(&id).unwrap()
        };
        for (k, v) in map {
            let key = k.as_str().unwrap();
            match key {
                "foreground" => { attr.foreground = v.as_u64().map(|c| to_sdl_color(c)); }
                "background" => { attr.background = v.as_u64().map(|c| to_sdl_color(c)); }
                "special" => { attr.special = v.as_u64().map(|c| to_sdl_color(c)); }
                "blend" => { attr.blend = v.as_u64().unwrap() as u8; }
                "reverse" => { attr.reverse = v.as_bool().unwrap(); }
                "italic" => { attr.italic = v.as_bool().unwrap(); }
                "bold" => { attr.bold = v.as_bool().unwrap(); }
                "strikethrough" => { attr.strikethrough = v.as_bool().unwrap(); }
                "underline" => { attr.underline = v.as_bool().unwrap(); }
                "undercurl" => { attr.undercurl = v.as_bool().unwrap(); }
                _ => { println!("Unsupported hl attr key {} in {:?}", key, map); }
            }
        }
    }
}

fn do_redraw(state: &mut NvimState, args: Drain<'_, Value>) {
    for update_events in args {
        if let Value::Array(update_events) = update_events {
            let mut update_events_iter = update_events.into_iter();
            if let Some(event_name) = update_events_iter.next() {
                if let Some(str) = event_name.as_str() {
                    for events in update_events_iter {
                        let arr = events.as_array();
                        match str {
                            "default_colors_set" => {
                                let mut args = arr.unwrap().into_iter();
                                state.default_colors_set(
                                    args.next().map(|v| v.as_u64().unwrap()),
                                    args.next().map(|v| v.as_u64().unwrap()),
                                    args.next().map(|v| v.as_u64().unwrap())
                                );
                            }
                            "grid_clear" => {
                                let mut args = arr.unwrap().into_iter();
                                state.grid_clear(
                                    args.next().unwrap().as_u64().unwrap() as NvimGridId,
                                );
                            }
                            "grid_cursor_goto" => {
                                let mut args = arr.unwrap().into_iter();
                                state.grid_cursor_goto(
                                    args.next().unwrap().as_u64().unwrap() as NvimGridId,
                                    args.next().unwrap().as_u64().unwrap() as NvimRow,
                                    args.next().unwrap().as_u64().unwrap() as NvimColumn,
                                );
                            }
                            "grid_line" => {
                                let mut args = arr.unwrap().into_iter();
                                let grid = args.next().unwrap().as_u64().unwrap() as NvimGridId;
                                let row = args.next().unwrap().as_u64().unwrap() as NvimRow;
                                let col_start = args.next().unwrap().as_u64().unwrap() as NvimColumn;
                                if let Value::Array(cells) = args.next().unwrap() {
                                    state.grid_line(
                                        grid,
                                        row,
                                        col_start,
                                        &cells,
                                    );
                                }
                            }
                            "grid_resize" => {
                                let mut args = arr.unwrap().into_iter();
                                state.grid_resize(
                                    args.next().unwrap().as_u64().unwrap() as NvimGridId,
                                    args.next().unwrap().as_u64().unwrap() as NvimWidth,
                                    args.next().unwrap().as_u64().unwrap() as NvimHeight,
                                );
                            }
                            "grid_scroll" => {
                                let mut args = arr.unwrap().into_iter();
                                state.grid_scroll(
                                    args.next().unwrap().as_u64().unwrap() as NvimGridId,
                                    args.next().unwrap().as_u64().unwrap() as NvimRow,
                                    args.next().unwrap().as_u64().unwrap() as NvimRow,
                                    args.next().unwrap().as_u64().unwrap() as NvimColumn,
                                    args.next().unwrap().as_u64().unwrap() as NvimColumn,
                                    args.next().unwrap().as_i64().unwrap(),
                                    args.next().unwrap().as_i64().unwrap(),
                                );
                            }
                            "hl_attr_define" => {
                                let mut args = arr.unwrap().into_iter();
                                state.hl_attr_define(
                                    args.next().unwrap().as_u64().unwrap(),
                                    args.next().unwrap().as_map().unwrap()
                                );
                            }
                            "win_viewport" => {}, // Don't care about win_viewport, stop spamming about it!
                            _ => {
                                // println!("Unhandled {}", str);
                            }
                        }
                    }
                } else {
                    eprintln!("Found non-str event name!");
                }
            } else {
                eprintln!("No event name!");
            }
        } else {
            eprintln!("Unsupported event type {:?}", update_events);
        }
    }
}

pub fn main() -> Result<(), String> {
    env::remove_var("NVIM_LISTEN_ADDRESS");
    let session = Session::new_child().unwrap();
    let mut nvim = Neovim::new(session);
    let mut state = NvimState::new();
    let chan = nvim.session.start_event_loop_channel();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut window_rectangle = Rect::new(0, 0, 800, 600);
    let window = video_subsystem
        .window("Nwin", window_rectangle.width(), window_rectangle.height())
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    canvas.present();

    {
        let window = canvas.window_mut();
        let size = window.size();
        window_rectangle.set_width(size.0);
        window_rectangle.set_height(size.1);
    }

    let texture_creator = canvas.texture_creator();

    let font = ttf_context.load_font(
        "/home/me/downloads/NotoSansMono/NotoSansMono-Regular.ttf",
        16
    )?;

    let surface = font
        .render("A")
        .blended(Color::RGBA(255, 0, 0, 255))
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let t = texture.query();
    let font_width = t.width;
    let font_height = t.height;

    // Create texture atlas
    // Start with something that can host ~128 chars * 10 highlights
    let atlas_width = font_width * 1000;
    let mut atlas_texture = texture_creator.create_texture_target(None,
        atlas_width,
        font_height,
    ).unwrap();
    let mut atlas_index : HashMap<AtlasIndexKey, (i32, u32)> = HashMap::new();
    let mut next_atlas_slot = font_width as i32;

    nvim.ui_attach(
        (window_rectangle.width() / font_width).into(),
        (window_rectangle.height() / font_height).into(),
        UiAttachOptions::new()
        .set_rgb(true)
        .set_linegrid_external(true)
    ).unwrap();

    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;

    let mut big_texture = texture_creator.create_texture_target(None,
        window_rectangle.width(),
        window_rectangle.height(),
    ).unwrap();
    let mut big_texture_copy = texture_creator.create_texture_target(None,
        window_rectangle.width(),
        window_rectangle.height(),
    ).unwrap();


    let mut cursor_rect = Rect::new(0, 0, 0, 0);
    let mut redraw_messages = VecDeque::new();

    'running: loop {
        let now = Instant::now();
        // 1) Process events from neovim
        while let Ok((str, messages)) = chan.try_recv() {
            if str == "redraw" {
                // Copy messages into the vecdequeue, remember position of last flush if there's
                // one.
                let len = messages.len();
                let mut i = 0;
                let mut last_flush_position = None;
                for msg in messages {
                    if let Value::Array(ref events) = msg {
                        if let Some(str) = events.into_iter().next() {
                            if let Some(str) = str.as_str() {
                                if str == "flush" {
                                    last_flush_position = Some(len - i);
                                }
                            }
                        }
                    }
                    i += 1;
                    redraw_messages.push_back(msg);
                }
                if let Some(pos) = last_flush_position {
                    do_redraw(&mut state, redraw_messages.drain(0..redraw_messages.len() - pos));
                }
            } else {
                eprintln!("Unexpected message: {}", str);
            }
        }

        // 2) React to window size changes
        {
            // Update the window title.
            let window = canvas.window_mut();

            let new_size = window.size();
            if new_size.0 != window_rectangle.width() || new_size.1 != window_rectangle.height() {
                // Inform neovim of the change
                nvim.ui_try_resize_grid(1,
                    (new_size.0 / font_width).into(),
                    (new_size.1 / font_height).into(),
                ).unwrap();
                // Change size of rendering texture
                let old_texture = big_texture;
                big_texture = texture_creator.create_texture_target(
                    None,
                    new_size.0,
                    new_size.1).unwrap();
                big_texture_copy = texture_creator.create_texture_target(
                    None,
                    new_size.0,
                    new_size.1).unwrap();
                // Copy old rendering texture to new one
                let r = Rect::new(
                    0,
                    0,
                    std::cmp::min(new_size.1, window_rectangle.width()),
                    std::cmp::min(new_size.0, window_rectangle.height())
                );
                canvas.with_texture_canvas(&mut big_texture, |canvas| {
                    canvas.copy(&old_texture, r, r).unwrap();
                }).unwrap();
                // Remember the size of the new window
                window_rectangle.set_width(new_size.0);
                window_rectangle.set_height(new_size.1);
            }

        }

        // 3) Redraw grid damages
        if let Some(default_hl) = state.hl_attrs.get(&0) {
            let default_bg = default_hl.background;
            let default_fg = default_hl.foreground;
            let grid = state.grids.get_mut(&1).unwrap();
            for d in &grid.damages {
                if let Damage::Cell{ row, column, width, height } = d {
                    let damage_top = *row;
                    let mut damage_bottom = row + height;
                    if damage_bottom > grid.get_height() {
                        damage_bottom = grid.get_height();
                    }
                    for current_row in damage_top .. damage_bottom {
                        let damage_left = *column;
                        let mut damage_right = column + width;
                        if damage_right > grid.get_width() {
                            damage_right = grid.get_width();
                        }
                        for current_column in damage_left .. damage_right {
                            let char_id = grid.chars[current_row][current_column].or_else(||Some(0 as char)).unwrap() as u64;
                            let attr_id = grid.colors[current_row][current_column];
                            let atlas_key = ((attr_id & (2u64.pow(32) - 1)) << 32)
                                | (char_id & (2u64.pow(32) - 1));
                            if let None = atlas_index.get(&atlas_key) {
                                let hl_attr = state.hl_attrs.get(&attr_id).unwrap();
                                canvas.with_texture_canvas(&mut atlas_texture, |canvas| {
                                    let bg = hl_attr.background.or_else(||default_bg).unwrap();
                                    let fg = hl_attr.foreground.or_else(||default_fg).unwrap();
                                    canvas.set_draw_color(bg);

                                    if let Some(char) = grid.chars[current_row][current_column] {
                                        let surface = font
                                            .render(&char.to_string())
                                            .blended(fg)
                                            .map_err(|e| e.to_string()).unwrap();
                                        let texture = texture_creator
                                            .create_texture_from_surface(&surface)
                                            .map_err(|e| e.to_string()).unwrap();
                                        let t = texture.query();
                                        let cell_rect = Rect::new(
                                            next_atlas_slot,
                                            0,
                                            t.width,
                                            t.height,
                                        );
                                        canvas.fill_rect(cell_rect).unwrap();
                                        canvas.copy(&texture, None, cell_rect).unwrap();
                                        atlas_index.insert(atlas_key, (next_atlas_slot, t.width));
                                        next_atlas_slot += t.width as i32;
                                    } else {
                                        let cell_rect = Rect::new(
                                            next_atlas_slot,
                                            0,
                                            font_width,
                                            font_height,
                                        );
                                        canvas.fill_rect(cell_rect).unwrap();
                                        atlas_index.insert(atlas_key, (next_atlas_slot, font_width));
                                        next_atlas_slot += font_width as i32;
                                    }
                                }).unwrap();
                            }
                            let (pos, width) = atlas_index.get(&atlas_key).unwrap();
                            canvas.with_texture_canvas(&mut big_texture, |canvas| {
                                let from = Rect::new(*pos, 0, *width, font_height);
                                let to = Rect::new(
                                    (current_column as i32) * (font_width as i32),
                                    (current_row as i32) * (font_height as i32),
                                    *width,
                                    font_height,
                                );
                                canvas.copy(&atlas_texture, from, to).unwrap();
                            }).unwrap();
                        }
                    }
                } else if let Damage::VerticalScroll{ from, to, height } = d {
                    canvas.with_texture_canvas(&mut big_texture_copy, |canvas| {
                        canvas.copy(&big_texture, None, None).unwrap();
                    }).unwrap();
                    canvas.with_texture_canvas(&mut big_texture, |canvas| {
                        let f = Rect::new(
                            0,
                            (*from as i32) * (font_height as i32),
                            window_rectangle.width(),
                            (*height as u32) * (font_height as u32),
                        );
                        let t = Rect::new(
                            0,
                            (*to as i32) * (font_height as i32),
                            window_rectangle.width(),
                            (*height as u32) * (font_height as u32)
                        );
                        canvas.copy(&big_texture_copy, f, t).unwrap();
                    }).unwrap();
                }
            }
            canvas.copy(&big_texture, window_rectangle, window_rectangle).unwrap();

            let (row, column) = grid.get_cursor_pos();
            let attr_id = grid.colors[row as usize][column as usize];
            if let Some(hl_attr) = state.hl_attrs.get(&attr_id) {
                canvas.set_draw_color(hl_attr.foreground.or_else(||default_fg).unwrap());
                cursor_rect.set_x((column as i32) * (font_width as i32));
                cursor_rect.set_y((row as i32) * (font_height as i32));
                cursor_rect.set_width(font_width);
                cursor_rect.set_height(font_height);
                canvas.fill_rect(cursor_rect).unwrap();
            }
            canvas.present();
            grid.damages.truncate(0);
        }

        // Use the time we have left before having to display the next frame to read events from
        // ui and forward them to neovim if necessary.
        let mut time_left = (1000/60) - i64::try_from(now.elapsed().as_millis()).unwrap() - 1;
        if time_left < 1 {
            time_left = 0
        }
        let mut input_string = "".to_owned();
        for event in event_pump.wait_timeout_iter(time_left as u32) {
            match event {
                Event::Quit { .. } => { nvim.quit_no_save().unwrap(); break 'running; },
                Event::KeyDown { .. } => {
                    if let Some(str) = keys::nvim_event_representation(event) {
                        input_string.push_str(&str);
                    }
                },
                Event::TextInput { text: s, .. } => {
                    for c in s.chars() {
                        // NOTE: We ignore space because it has a non-literal repr and it's better
                        // to have it go through the keydown nvim.input, in order to be able to
                        // handle both <Space> and <S-Space> (we can't tell <S-Space> from a
                        // TextInput event).
                        if c != ' ' {
                            if let Some(s) = keys::nvim_char_representation(c) {
                                input_string.push_str(s);
                            } else {
                                input_string.push_str(&c.to_string());
                            }
                        }
                    }
                }
                _ => {},
            }
        }
        if input_string != "" {
            nvim.input(&input_string).unwrap();
        }
    }

    Ok(())
}

