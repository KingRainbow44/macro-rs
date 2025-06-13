use std::fmt::Formatter;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use device_query::{DeviceEvents, DeviceEventsHandler, DeviceQuery, DeviceState, MouseButton};
use enigo::{Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MouseMoveAction {
    delta_x: i32,
    delta_y: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MouseButtonAction {
    button: MouseButton,
    pressed: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeyAction {
    key: String,
    pressed: bool
}

/// A user action represents the types of actions that can be
/// recorded and the data associated with them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum UserAction {
    MouseMove(MouseMoveAction),
    MouseButton(MouseButtonAction),
    Key(KeyAction)
}

/// A macro action that includes the type of action and the
/// offset in time when the action occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MacroAction {
    action: UserAction,
    offset: u64
}

pub struct MacroGuard {
    // Store all variables as type-erased boxes
    _guards: Vec<Box<dyn std::any::Any>>,
}

/// The `MacroGuard` is used to keep the references to the
/// callback handlers for `device_query` in scope.
///
/// If the `MacroGuard` instance is dropped, the recording
/// will stop and the callbacks will no longer be active.
impl MacroGuard {
    fn new() -> Self {
        Self {
            _guards: Vec::new(),
        }
    }

    pub(crate) fn keep_alive<T: 'static>(mut self, value: T) -> Self {
        self._guards.push(Box::new(value));
        self
    }
}

/// The metadata of a `Macro` includes:
/// - The end timestamp of the macro
/// - The initial cursor starting position
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MacroMetadata {
    pub(crate) end: u64,
    pub(crate) cursor_pos: (i32, i32)
}

/// The `Macro` struct represents a series of actions taken by
/// the user such as key presses, mouse clicks, and mouse movements.
///
/// This struct is serializable and can be deserialized later to
/// replay actions.
#[derive(Debug)]
pub struct Macro {
    enigo: Enigo,

    start_time: Arc<Mutex<Instant>>,
    is_recording: Arc<Mutex<bool>>,
    last_pos: Arc<Mutex<(i32, i32)>>,

    actions: Arc<Mutex<Vec<MacroAction>>>,
    metadata: Arc<Mutex<MacroMetadata>>
}

impl Macro {
    /// Creates a new macro instance.
    pub fn new() -> Self {
        Macro {
            enigo: Enigo::new(&Settings::default()).unwrap(),
            start_time: Arc::new(Mutex::new(Instant::now())),
            is_recording: Arc::new(Mutex::new(false)),
            last_pos: Arc::new(Mutex::new((0, 0))),
            actions: Arc::new(Mutex::new(vec![])),
            metadata: Arc::new(Mutex::new(MacroMetadata::default()))
        }
    }

    /// Starts the recording of user actions.
    ///
    /// The returned guard must be held to keep the recording active.
    pub fn record(&self) -> MacroGuard {
        // Mark state as recording.
        *self.is_recording.lock().unwrap() = true;

        // Clear existing actions.
        self.actions.lock().unwrap().clear();

        let start = Instant::now();
        let listener = DeviceEventsHandler::new(Duration::from_micros(500)).unwrap();

        // Set the starting cursor position.
        let state = DeviceState::new();
        let (x, y) = state.get_mouse().coords;
        // Store the initial cursor position in the metadata.
        self.metadata.lock().unwrap().cursor_pos = (x, y);
        *self.last_pos.lock().unwrap() = (x, y);

        let key_up = self.actions.clone();
        let key_down = self.actions.clone();
        let mouse_up = self.actions.clone();
        let mouse_down = self.actions.clone();
        let mouse_move = self.actions.clone();

        let last_pos = self.last_pos.clone();

        // Start listening for device events.
        let key_up_guard = listener.on_key_up(move |key| {
            // Record the key up action.
            key_up.lock().unwrap().push(MacroAction {
                offset: Instant::now().time_since(start),
                action: UserAction::Key(KeyAction { key: key.to_string(), pressed: false })
            })
        });

        let key_down_guard = listener.on_key_down(move |key| {
            // Record the key down action.
            key_down.lock().unwrap().push(MacroAction {
                offset: Instant::now().time_since(start),
                action: UserAction::Key(KeyAction { key: key.to_string(), pressed: true })
            })
        });

        let mouse_up_guard = listener.on_mouse_up(move |button| {
            // Record the mouse button up action.
            mouse_up.lock().unwrap().push(MacroAction {
                offset: Instant::now().time_since(start),
                action: UserAction::MouseButton(MouseButtonAction { button: *button, pressed: false })
            })
        });

        let mouse_down_guard = listener.on_mouse_down(move |button| {
            // Record the mouse button down action.
            mouse_down.lock().unwrap().push(MacroAction {
                offset: Instant::now().time_since(start),
                action: UserAction::MouseButton(MouseButtonAction { button: *button, pressed: true })
            })
        });

        let mouse_move_guard = listener.on_mouse_move(move |position| {
            // Record the mouse move action.
            let mut last_pos = last_pos.lock().unwrap();
            // Calculate the delta from the last position.
            let (x, y) = position;
            let (delta_x, delta_y) = (x - last_pos.0, y - last_pos.1);
            // Update the last position.
            *last_pos = (*x, *y);

            mouse_move.lock().unwrap().push(MacroAction {
                offset: Instant::now().time_since(start),
                action: UserAction::MouseMove(MouseMoveAction { delta_x, delta_y })
            });
        });

        MacroGuard::new()
            .keep_alive(key_up_guard)
            .keep_alive(key_down_guard)
            .keep_alive(mouse_up_guard)
            .keep_alive(mouse_down_guard)
            .keep_alive(mouse_move_guard)
    }

    /// Stops the macro recording.
    pub fn stop_recording(&self) {
        // This will stop any threads from holding on
        // to the recording state if they use `is_recording()`.
        *self.is_recording.lock().unwrap() = false;

        // Set the end time of the macro.
        let start_time = *self.start_time.lock().unwrap();
        self.metadata.lock().unwrap().end = Instant::now().time_since(start_time);
    }

    /// Checks if a macro is currently being recorded.
    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }

    /// Plays any stored macro actions.
    ///
    /// This method will block until all actions have been played back.
    pub fn playback(&mut self) {
        let start = Instant::now();
        let metadata = self.metadata.lock().unwrap();
        let actions = self.actions.lock().unwrap();

        // Move the cursor to the initial position.
        let (x, y) = metadata.cursor_pos;
        self.enigo.move_mouse(x, y, Coordinate::Abs).unwrap();

        loop {
            let offset = Instant::now().time_since(start);

            // Check if the macro is over.
            if offset >= metadata.end {
                // Stop playback if the end time has been reached.
                break;
            }

            // Get the actions to play back.
            for action in actions.iter()
                .filter(|a| a.offset.eq(&offset)) {
                match &action.action {
                    UserAction::MouseMove(mouse) => {
                        self.enigo.move_mouse(mouse.delta_x, mouse.delta_y, Coordinate::Rel).unwrap();
                    }
                    UserAction::MouseButton(mouse) => {
                        let direction = if mouse.pressed {
                            Direction::Press
                        } else {
                            Direction::Release
                        };
                        let button = match mouse.button {
                            1 => Button::Left,
                            2 => Button::Right,
                            3 => Button::Middle,
                            4 => Button::Back,
                            5 => Button::Forward,
                            _ => {
                                eprintln!("Unknown mouse button: {}", mouse.button);
                                continue;
                            }
                        };

                        self.enigo.button(button, direction).unwrap();
                    }
                    UserAction::Key(key) => {
                        let direction = if key.pressed {
                            Direction::Press
                        } else {
                            Direction::Release
                        };

                        if let Some(key) = utils::remap(&key.key) {
                            self.enigo.key(key, direction).unwrap();
                        }
                    }
                }
            }

            // Wait for the next millisecond.
            sleep(Duration::from_micros(500));
        }
    }

    /// Saves this macro to the file system.
    #[cfg(feature = "save")]
    pub fn save<S: AsRef<str>>(&self, path: S) {
        if let Some(parent) = std::path::Path::new(path.as_ref()).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).expect("Failed to create directory");
            }
        }

        let content = serde_json::to_string(self)
            .expect("Failed to serialize macro");
        if let Err(e) = std::fs::write(path.as_ref(), content) {
            eprintln!("Failed to write macro to file: {}", e);
        }
    }
}

impl Clone for Macro {
    fn clone(&self) -> Self {
        Macro {
            enigo: Enigo::new(&Settings::default()).unwrap(),
            metadata: self.metadata.clone(),
            start_time: self.start_time.clone(),
            is_recording: self.is_recording.clone(),
            actions: self.actions.clone(),
            last_pos: self.last_pos.clone(),
        }
    }
}

impl Serialize for Macro {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let actions = self.actions.lock().unwrap();
        let metadata = self.metadata.lock().unwrap();
        let mut state = serializer.serialize_struct("Macro", 2)?;
        state.serialize_field("actions", &*actions)?;
        state.serialize_field("metadata", &*metadata)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Macro {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        deserializer.deserialize_struct("Macro", &["actions", "metadata"], MacroVisitor)
    }
}

struct MacroVisitor;

impl<'de> Visitor<'de> for MacroVisitor {
    type Value = Macro;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a macro with actions and metadata")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut actions = None;
        let mut metadata = None;

        while let Some(key) = seq.next_element::<String>()? {
            match key.as_str() {
                "actions" => {
                    actions = Some(seq.next_element::<Vec<MacroAction>>()?);
                }
                "metadata" => {
                    metadata = Some(seq.next_element::<MacroMetadata>()?);
                }
                _ => return Err(serde::de::Error::unknown_field(&key, &["actions", "metadata"])),
            }
        }

        let actions = actions
            .ok_or_else(|| serde::de::Error::missing_field("actions"))?;
        let metadata = metadata
            .ok_or_else(|| serde::de::Error::missing_field("metadata"))?;

        Ok(Macro {
            enigo: Enigo::new(&Settings::default()).unwrap(),
            start_time: Arc::new(Mutex::new(Instant::now())),
            is_recording: Arc::new(Mutex::new(false)),
            last_pos: Arc::new(Mutex::new((0, 0))),
            actions: Arc::new(Mutex::new(actions.unwrap())),
            metadata: Arc::new(Mutex::new(metadata.unwrap())),
        })
    }
}

trait TimeSince {
    /// Returns the time in milliseconds since the given start time.
    fn time_since(&self, start: Instant) -> u64;
}

impl TimeSince for Instant {
    fn time_since(&self, start: Instant) -> u64 {
        self.duration_since(start).as_millis() as u64
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_macro() {
        sleep(Duration::from_secs(1));

        let mut towa = Macro::new();
        let thread_towa = towa.clone();
        std::thread::spawn(move || {
            let _guard = thread_towa.record();
            while thread_towa.is_recording() {
                // Busy wait for the recording to finish.
            }
        });

        sleep(Duration::from_secs(3));
        towa.stop_recording();

        sleep(Duration::from_secs(2));
        towa.playback();
    }

    #[test]
    fn serialize_macro() {
        sleep(Duration::from_secs(1));

        let towa = Macro::new();
        let thread_towa = towa.clone();
        std::thread::spawn(move || {
            let _guard = thread_towa.record();
            while thread_towa.is_recording() {
                // Busy wait for the recording to finish.
            }
        });

        sleep(Duration::from_secs(3));
        towa.stop_recording();

        let serialized = serde_json::to_string(&towa)
            .expect("failed to serialize macro");
        println!("macro: {:?}", serialized);
    }

    #[test]
    #[cfg(feature = "save")]
    fn save_macro() {
        sleep(Duration::from_secs(1));

        let towa = Macro::new();
        let thread_towa = towa.clone();
        std::thread::spawn(move || {
            let _guard = thread_towa.record();
            while thread_towa.is_recording() {
                // Busy wait for the recording to finish.
            }
        });

        sleep(Duration::from_secs(3));
        towa.stop_recording();

        // Save the macro to a file.
        towa.save("test_macro.json");
    }
}