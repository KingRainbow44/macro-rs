use std::str::FromStr;
use device_query::Keycode;
use enigo::Key;

/// Remaps a key name from `device_query` to `enigo`'s `Key`.
/// 
/// Taken from: https://github.com/lopo12123/toca/blob/master/src/mapper.rs
pub(crate) fn remap(key_name: &String) -> Option<Key> {
    // Parse the key name into a `Keycode`.
    let Ok(keycode) = Keycode::from_str(key_name.as_str()) else {
        return None;
    };

    match keycode {
        // F1-F12
        Keycode::F1 => Some(Key::F1),
        Keycode::F2 => Some(Key::F2),
        Keycode::F3 => Some(Key::F3),
        Keycode::F4 => Some(Key::F4),
        Keycode::F5 => Some(Key::F5),
        Keycode::F6 => Some(Key::F6),
        Keycode::F7 => Some(Key::F7),
        Keycode::F8 => Some(Key::F8),
        Keycode::F9 => Some(Key::F9),
        Keycode::F10 => Some(Key::F10),
        Keycode::F11 => Some(Key::F11),
        Keycode::F12 => Some(Key::F12),
        // 0-9
        Keycode::Key0 => Some(Key::Num0),
        Keycode::Key1 => Some(Key::Num1),
        Keycode::Key2 => Some(Key::Num2),
        Keycode::Key3 => Some(Key::Num3),
        Keycode::Key4 => Some(Key::Num4),
        Keycode::Key5 => Some(Key::Num5),
        Keycode::Key6 => Some(Key::Num6),
        Keycode::Key7 => Some(Key::Num7),
        Keycode::Key8 => Some(Key::Num8),
        Keycode::Key9 => Some(Key::Num9),
        // A-Z
        Keycode::A => Some(Key::A),
        Keycode::B => Some(Key::B),
        Keycode::C => Some(Key::C),
        Keycode::D => Some(Key::D),
        Keycode::E => Some(Key::E),
        Keycode::F => Some(Key::F),
        Keycode::G => Some(Key::G),
        Keycode::H => Some(Key::H),
        Keycode::I => Some(Key::I),
        Keycode::J => Some(Key::J),
        Keycode::K => Some(Key::K),
        Keycode::L => Some(Key::L),
        Keycode::M => Some(Key::M),
        Keycode::N => Some(Key::N),
        Keycode::O => Some(Key::O),
        Keycode::P => Some(Key::P),
        Keycode::Q => Some(Key::Q),
        Keycode::R => Some(Key::R),
        Keycode::S => Some(Key::S),
        Keycode::T => Some(Key::T),
        Keycode::U => Some(Key::U),
        Keycode::V => Some(Key::V),
        Keycode::W => Some(Key::W),
        Keycode::X => Some(Key::X),
        Keycode::Y => Some(Key::Y),
        Keycode::Z => Some(Key::Z),
        // from left to right, from top to bottom
        Keycode::Escape => Some(Key::Escape),
        Keycode::Tab => Some(Key::Tab),
        Keycode::CapsLock => Some(Key::CapsLock),
        Keycode::LShift | Keycode::RShift => Some(Key::Shift),
        Keycode::LControl | Keycode::RControl => Some(Key::Control),
        Keycode::LAlt | Keycode::RAlt => Some(Key::Alt),
        Keycode::Space => Some(Key::Space),
        Keycode::Up => Some(Key::UpArrow),
        Keycode::Right => Some(Key::RightArrow),
        Keycode::Down => Some(Key::DownArrow),
        Keycode::Left => Some(Key::LeftArrow),
        Keycode::Enter => Some(Key::Return),
        Keycode::Backspace => Some(Key::Backspace),
        // Keycode::Insert => None,
        Keycode::Delete => Some(Key::Delete),
        Keycode::Home => Some(Key::Home),
        Keycode::PageUp => Some(Key::PageUp),
        Keycode::PageDown => Some(Key::PageDown),
        Keycode::End => Some(Key::End),
        // belows have passed the simulate test
        Keycode::Grave => Some(Key::Unicode('`')),
        Keycode::Minus | Keycode::NumpadSubtract => Some(Key::Unicode('-')),
        Keycode::Equal => Some(Key::Unicode('=')),
        Keycode::LeftBracket => Some(Key::Unicode('[')),
        Keycode::RightBracket => Some(Key::Unicode(']')),
        Keycode::Comma => Some(Key::Unicode(',')),
        Keycode::Dot => Some(Key::Unicode('.')),
        Keycode::Semicolon => Some(Key::Unicode(';')),
        Keycode::Apostrophe => Some(Key::Unicode('\'')),
        Keycode::Slash | Keycode::NumpadDivide => Some(Key::Divide),
        Keycode::BackSlash => Some(Key::Unicode('\\')),
        // belows have no exact target in Enigo but can also use in typing
        Keycode::Numpad0 => Some(Key::Numpad0),
        Keycode::Numpad1 => Some(Key::Numpad1),
        Keycode::Numpad2 => Some(Key::Numpad2),
        Keycode::Numpad3 => Some(Key::Numpad3),
        Keycode::Numpad4 => Some(Key::Numpad4),
        Keycode::Numpad5 => Some(Key::Numpad5),
        Keycode::Numpad6 => Some(Key::Numpad6),
        Keycode::Numpad7 => Some(Key::Numpad7),
        Keycode::Numpad8 => Some(Key::Numpad8),
        Keycode::Numpad9 => Some(Key::Numpad9),
        _ => None
    }
}