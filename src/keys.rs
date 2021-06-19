extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;

// Issues:
// 1) `/` is actually `:`+`Shift`. We need to catch this as returning `<S-:>` would result in `:/`
//    being inserted, which is wrong.
// 2) `>` is actually `<`+`Shift`, where `<` has a non-literal rep.
// 3) ` ` has a non-literal repr, but also a textinput event. Solution: ignore textinput event, as
//    it can't handle stuff like <S-Space> while keydown can. ???: Are there other keys in a
//    similar situation?
// 4) RALT is ALTGR - frequently used to produce alternative characters. Results in things like
//    <M-l>λ being inserted for a single <RALT-l>. Solution: use config to decide whether ralt
//    should be listened to? Ignore ralt for now.
fn with_mod(s: &str, m: Mod) -> Option<String> {
    let has_gui = (m & Mod::LGUIMOD != Mod::NOMOD) || (m & Mod::RGUIMOD != Mod::NOMOD);
    let has_ctrl = (m & Mod::LCTRLMOD != Mod::NOMOD) || (m & Mod::RCTRLMOD != Mod::NOMOD);
    let has_alt = m & Mod::LALTMOD != Mod::NOMOD/* || (m & Mod::RALTMOD != Mod::NOMOD)*/;
    let has_non_shift_mod = has_gui || has_ctrl || has_alt;
    let has_literal_repr = s.chars().next().unwrap() != '<';

    // Take care of 1)
    if has_literal_repr && !has_non_shift_mod {
        return None;
    }
    if s == "<LT>" && !has_non_shift_mod {
        return None;
    }
    let mut result = s.to_string();
    if has_literal_repr {
        result.insert_str(0, "<");
        result.insert_str(2, ">");
    }
    let shifted = (m & Mod::LSHIFTMOD) != Mod::NOMOD || (m & Mod::RSHIFTMOD) != Mod::NOMOD;
    if (shifted && !(m & Mod::CAPSMOD != Mod::NOMOD))
        || (!shifted && (m & Mod::CAPSMOD != Mod::NOMOD))
    {
        result.insert_str(1, "S-");
    }
    if has_gui {
        result.insert_str(1, "D-");
    }
    if has_ctrl {
        result.insert_str(1, "C-");
    }
    if has_alt {
        result.insert_str(1, "A-");
    }
    Some(result)
}

pub fn nvim_char_representation(c: char) -> Option<&'static str> {
    match c {
        '<' => Some("<LT>"),
        ' ' => Some("<Space>"),
        _ => None,
    }
}

pub fn nvim_event_representation(event: Event) -> Option<String> {
    if let Event::KeyDown {
        keycode: Some(k),
        keymod: m,
        ..
    } = event
    {
        match k {
            // Alpha
            Keycode::A => with_mod("a", m),
            Keycode::B => with_mod("b", m),
            Keycode::C => with_mod("c", m),
            Keycode::D => with_mod("d", m),
            Keycode::E => with_mod("e", m),
            Keycode::F => with_mod("f", m),
            Keycode::G => with_mod("g", m),
            Keycode::H => with_mod("h", m),
            Keycode::I => with_mod("i", m),
            Keycode::J => with_mod("j", m),
            Keycode::K => with_mod("k", m),
            Keycode::L => with_mod("l", m),
            Keycode::M => with_mod("m", m),
            Keycode::N => with_mod("n", m),
            Keycode::O => with_mod("o", m),
            Keycode::P => with_mod("p", m),
            Keycode::Q => with_mod("q", m),
            Keycode::R => with_mod("r", m),
            Keycode::S => with_mod("s", m),
            Keycode::T => with_mod("t", m),
            Keycode::U => with_mod("u", m),
            Keycode::V => with_mod("v", m),
            Keycode::W => with_mod("w", m),
            Keycode::X => with_mod("x", m),
            Keycode::Y => with_mod("y", m),
            Keycode::Z => with_mod("z", m),
            // Numerical
            Keycode::Num0 => with_mod("0", m),
            Keycode::Num1 => with_mod("1", m),
            Keycode::Num2 => with_mod("2", m),
            Keycode::Num3 => with_mod("3", m),
            Keycode::Num4 => with_mod("4", m),
            Keycode::Num5 => with_mod("5", m),
            Keycode::Num6 => with_mod("6", m),
            Keycode::Num7 => with_mod("7", m),
            Keycode::Num8 => with_mod("8", m),
            Keycode::Num9 => with_mod("9", m),
            // Single-char
            Keycode::Ampersand => with_mod("&", m),
            Keycode::Asterisk => with_mod("*", m),
            Keycode::At => with_mod("@", m),
            Keycode::Backquote => with_mod("`", m),
            Keycode::Backslash => with_mod("\\", m),
            Keycode::Caret => with_mod("^", m),
            Keycode::Colon => with_mod(":", m),
            Keycode::Comma => with_mod(",", m),
            Keycode::Dollar => with_mod("$", m),
            Keycode::Equals => with_mod("=", m),
            Keycode::Exclaim => with_mod("!", m),
            Keycode::Greater => with_mod(">", m),
            Keycode::Hash => with_mod("#", m),
            Keycode::KpA => with_mod("a", m),
            Keycode::KpAmpersand => with_mod("&", m),
            Keycode::KpAt => with_mod("at", m),
            Keycode::KpB => with_mod("b", m),
            Keycode::KpC => with_mod("c", m),
            Keycode::KpColon => with_mod(":", m),
            Keycode::KpD => with_mod("D", m),
            Keycode::KpDblAmpersand => with_mod("&&", m),
            Keycode::KpDblVerticalBar => with_mod("||", m),
            Keycode::KpDecimal => with_mod(".", m),
            Keycode::KpE => with_mod("e", m),
            Keycode::KpExclam => with_mod("!", m),
            Keycode::KpF => with_mod("f", m),
            Keycode::KpGreater => with_mod(">", m),
            Keycode::KpHash => with_mod("#", m),
            Keycode::KpLeftBrace => with_mod("{", m),
            Keycode::KpLeftParen => with_mod("(", m),
            Keycode::KpPercent => with_mod("%", m),
            Keycode::KpPeriod => with_mod(".", m),
            Keycode::KpRightBrace => with_mod("}", m),
            Keycode::KpRightParen => with_mod(")", m),
            Keycode::KpVerticalBar => with_mod("|", m),
            Keycode::LeftBracket => with_mod("[", m),
            Keycode::LeftParen => with_mod("(", m),
            Keycode::Minus => with_mod("-", m),
            Keycode::Percent => with_mod("%", m),
            Keycode::Period => with_mod(".", m),
            Keycode::Plus => with_mod("+", m),
            Keycode::Question => with_mod("?", m),
            Keycode::Quote => with_mod("'", m),
            Keycode::Quotedbl => with_mod("\"", m),
            Keycode::RightBracket => with_mod("]", m),
            Keycode::RightParen => with_mod(")", m),
            Keycode::Semicolon => with_mod(";", m),
            Keycode::Slash => with_mod("/", m),
            Keycode::Underscore => with_mod("_", m),
            // Special-repr
            Keycode::AcHome => with_mod("<kHome>", m),
            Keycode::Backspace => with_mod("<BS>", m),
            Keycode::Delete => with_mod("<Del>", m),
            Keycode::Down => with_mod("<Down>", m),
            Keycode::End => with_mod("<End>", m),
            Keycode::Escape => with_mod("<Esc>", m),
            Keycode::F1 => with_mod("<F1>", m),
            Keycode::F2 => with_mod("<F2>", m),
            Keycode::F3 => with_mod("<F3>", m),
            Keycode::F4 => with_mod("<F4>", m),
            Keycode::F5 => with_mod("<F5>", m),
            Keycode::F6 => with_mod("<F6>", m),
            Keycode::F7 => with_mod("<F7>", m),
            Keycode::F8 => with_mod("<F8>", m),
            Keycode::F9 => with_mod("<F9>", m),
            Keycode::F10 => with_mod("<10>", m),
            Keycode::F11 => with_mod("<11>", m),
            Keycode::F12 => with_mod("<12>", m),
            Keycode::F13 => with_mod("<13>", m),
            Keycode::F14 => with_mod("<14>", m),
            Keycode::F15 => with_mod("<15>", m),
            Keycode::F16 => with_mod("<16>", m),
            Keycode::F17 => with_mod("<17>", m),
            Keycode::F18 => with_mod("<18>", m),
            Keycode::F19 => with_mod("<19>", m),
            Keycode::F20 => with_mod("<20>", m),
            Keycode::F21 => with_mod("<21>", m),
            Keycode::F22 => with_mod("<22>", m),
            Keycode::F23 => with_mod("<23>", m),
            Keycode::F24 => with_mod("<24>", m),
            Keycode::Help => with_mod("<Help>", m),
            Keycode::Home => with_mod("<Home>", m),
            Keycode::Insert => with_mod("<Insert>", m),
            Keycode::Kp0 => with_mod("<k0>", m),
            Keycode::Kp1 => with_mod("<k1>", m),
            Keycode::Kp2 => with_mod("<k2>", m),
            Keycode::Kp3 => with_mod("<k3>", m),
            Keycode::Kp4 => with_mod("<k4>", m),
            Keycode::Kp5 => with_mod("<k5>", m),
            Keycode::Kp6 => with_mod("<k6>", m),
            Keycode::Kp7 => with_mod("<k7>", m),
            Keycode::Kp8 => with_mod("<k8>", m),
            Keycode::Kp9 => with_mod("<k9>", m),
            Keycode::Kp00 => with_mod("<k00>", m),
            Keycode::Kp000 => with_mod("<k000>", m),
            Keycode::KpBackspace => with_mod("<BS>", m),
            Keycode::KpComma => with_mod("<kComma>", m),
            Keycode::KpDivide => with_mod("<kDivide>", m),
            Keycode::KpEnter => with_mod("<kEnter>", m),
            Keycode::KpEquals => with_mod("<kEquals>", m),
            Keycode::KpEqualsAS400 => with_mod("<kEquals>", m),
            Keycode::KpLess => with_mod("<LT>", m),
            Keycode::KpMinus => with_mod("<kMinus>", m),
            Keycode::KpMultiply => with_mod("<kMultiply>", m),
            Keycode::KpPlus => with_mod("<kPlus>", m),
            Keycode::Left => with_mod("<Left>", m),
            Keycode::Less => with_mod("<LT>", m),
            Keycode::PageDown => with_mod("<PageDown>", m),
            Keycode::PageUp => with_mod("<PageUp>", m),
            Keycode::Return => with_mod("<CR>", m),
            Keycode::Return2 => with_mod("<CR>", m),
            Keycode::Right => with_mod("<Right>", m),
            Keycode::Space => with_mod("<Space>", m),
            Keycode::Tab => with_mod("<Tab>", m),
            Keycode::Undo => with_mod("<Undo>", m),
            Keycode::Up => with_mod("<Up>", m),
            // No repr
            _ => None,
        }
    } else {
        None
    }
}
