#![cfg(feature = "editor")]

use egui::{pos2, vec2, Pos2};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::event::WindowEvent::*;

pub struct Plateform {
    raw_input: egui::RawInput,
    pointer_pos: Option<Pos2>,
    modifier_state: ModifiersState,
}

impl Plateform {
    pub fn new(window_size: (u32, u32)) -> Self {
        Self {
            raw_input: egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::default(),
                    vec2(
                        window_size.0 as f32,
                        window_size.1 as f32,
                    ),
                )),
                ..Default::default()
            },
            pointer_pos: None,
            modifier_state: ModifiersState::empty(),
        }
    }

    pub fn handle_resize(&mut self, size: (u32, u32)) {
        if size.0 == 0 || size.1 == 0 {
            return;
        }
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            vec2(size.0 as f32, size.1 as f32),
        ));
    }

    pub fn handle_mouse_event(&mut self, event: &winit::event::Event<()>) {
        match event {
            winit::event::Event::WindowEvent {
                window_id: _window_id,
                event,
            } => match event {
                MouseInput { state, button, .. } => {
                    if let winit::event::MouseButton::Other(..) = button {
                    } else {
                        // push event only if the cursor is inside the window
                        if let Some(pointer_pos) = self.pointer_pos {
                            self.raw_input.events.push(egui::Event::PointerButton {
                                pos: pointer_pos,
                                button: match button {
                                    winit::event::MouseButton::Left => egui::PointerButton::Primary,
                                    winit::event::MouseButton::Right => {
                                        egui::PointerButton::Secondary
                                    }
                                    winit::event::MouseButton::Middle => {
                                        egui::PointerButton::Middle
                                    }
                                    winit::event::MouseButton::Forward => egui::PointerButton::Extra1,
                                    winit::event::MouseButton::Back => egui::PointerButton::Extra2,
                                    winit::event::MouseButton::Other(_) => unreachable!(),
                                },
                                pressed: *state == winit::event::ElementState::Pressed,
                                modifiers: Default::default(),
                            });
                        }
                    }
                }
                MouseWheel { delta, .. } => {
                    let delta = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            let line_height = 8.0;
                            vec2(*x, *y) * line_height
                        }
                        winit::event::MouseScrollDelta::PixelDelta(delta) => {
                            vec2(delta.x as f32, delta.y as f32)
                        }
                    };
                    // The ctrl (cmd on macos) key indicates a zoom is desired.
                    if self.raw_input.modifiers.ctrl || self.raw_input.modifiers.command {
                        self.raw_input
                            .events
                            .push(egui::Event::Zoom((delta.y / 200.0).exp()));
                    } else {
                        self.raw_input.events.push(egui::Event::Scroll(delta));
                    }
                }
                CursorMoved { position, .. } => {
                    let pointer_pos = pos2(
                        position.x as f32,
                        position.y as f32,
                    );
                    self.pointer_pos = Some(pointer_pos);
                    self.raw_input
                        .events
                        .push(egui::Event::PointerMoved(pointer_pos));
                }
                CursorLeft { .. } => {
                    self.pointer_pos = None;
                    self.raw_input.events.push(egui::Event::PointerGone);
                }
                _ => {}
            },
            winit::event::Event::DeviceEvent { .. } => {}
            _ => {}
        }
    }

    pub fn handle_input_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            KeyboardInput { event, .. } => {
                let pressed = event.state == winit::event::ElementState::Pressed;
                let ctrl = self.raw_input.modifiers.ctrl;

                // Push character event
                let ch = event.text.as_ref();
                if let Some(ch) = ch {
                    if is_printable(ch.chars().next().unwrap())
                        && !self.raw_input.modifiers.ctrl
                        && !self.raw_input.modifiers.command
                    {
                        self.raw_input
                            .events
                            .push(egui::Event::Text(ch.to_string()));
                    }
                }

                // Copy, Cut, Handle key events
                match (pressed, ctrl, event.physical_key) {
                    (true, true, PhysicalKey::Code(KeyCode::KeyC)) => {
                        self.raw_input.events.push(egui::Event::Copy)
                    }
                    (true, true, PhysicalKey::Code(KeyCode::KeyX)) => {
                        self.raw_input.events.push(egui::Event::Cut)
                    }
                    _ => {
                        if let Some(key) = winit_to_egui_key_code(event.physical_key) {
                            self.raw_input.events.push(egui::Event::Key {
                                key,
                                physical_key: Some(key),
                                pressed,
                                modifiers: self.raw_input.modifiers,
                                repeat: false,
                            });
                        }
                    }
                }
            }
            ModifiersChanged(input) => {
                self.modifier_state = input.state();
                self.raw_input.modifiers = winit_to_egui_modifiers(input.state());
            }
            _ => {}
        }
    }

    /// Returns `true` if egui should handle the event exclusively. Check this to
    /// avoid unexpected interactions, e.g. a mouse click registering "behind" the UI.
    pub fn captures_event(&self, event: &winit::event::WindowEvent, context: &egui::Context) -> bool {
        match event {
            KeyboardInput { .. } | ModifiersChanged(_) => {
                context.wants_keyboard_input()
            }
            MouseWheel { .. } | MouseInput { .. } => context.wants_pointer_input(),
            CursorMoved { .. } => context.is_using_pointer(),
            _ => false,
        }
    }

    pub fn get_raw_input(&mut self) -> egui::RawInput {
        self.raw_input.take()
    }
}

#[inline]
fn winit_to_egui_key_code(key: winit::keyboard::PhysicalKey) -> Option<egui::Key> {
    use egui::Key as E;
    Some(match key {
        PhysicalKey::Code(KeyCode::KeyA) => E::A,
        PhysicalKey::Code(KeyCode::KeyB) => E::B,
        PhysicalKey::Code(KeyCode::KeyC) => E::C,
        PhysicalKey::Code(KeyCode::KeyD) => E::D,
        PhysicalKey::Code(KeyCode::KeyE) => E::E,
        PhysicalKey::Code(KeyCode::KeyF) => E::F,
        PhysicalKey::Code(KeyCode::KeyG) => E::G,
        PhysicalKey::Code(KeyCode::KeyH) => E::H,
        PhysicalKey::Code(KeyCode::KeyI) => E::I,
        PhysicalKey::Code(KeyCode::KeyJ) => E::J,
        PhysicalKey::Code(KeyCode::KeyK) => E::K,
        PhysicalKey::Code(KeyCode::KeyL) => E::L,
        PhysicalKey::Code(KeyCode::KeyM) => E::M,
        PhysicalKey::Code(KeyCode::KeyN) => E::N,
        PhysicalKey::Code(KeyCode::KeyO) => E::O,
        PhysicalKey::Code(KeyCode::KeyP) => E::P,
        PhysicalKey::Code(KeyCode::KeyQ) => E::Q,
        PhysicalKey::Code(KeyCode::KeyR) => E::R,
        PhysicalKey::Code(KeyCode::KeyS) => E::S,
        PhysicalKey::Code(KeyCode::KeyT) => E::T,
        PhysicalKey::Code(KeyCode::KeyU) => E::U,
        PhysicalKey::Code(KeyCode::KeyV) => E::V,
        PhysicalKey::Code(KeyCode::KeyW) => E::W,
        PhysicalKey::Code(KeyCode::KeyX) => E::X,
        PhysicalKey::Code(KeyCode::KeyY) => E::Y,
        PhysicalKey::Code(KeyCode::KeyZ) => E::Z,
        PhysicalKey::Code(KeyCode::Escape) => E::Escape,
        PhysicalKey::Code(KeyCode::Tab) => E::Tab,
        PhysicalKey::Code(KeyCode::Space) => E::Space,
        PhysicalKey::Code(KeyCode::Insert) => E::Insert,
        PhysicalKey::Code(KeyCode::Delete) => E::Delete,
        PhysicalKey::Code(KeyCode::Home) => E::Home,
        PhysicalKey::Code(KeyCode::End) => E::End,
        PhysicalKey::Code(KeyCode::PageUp) => E::PageUp,
        PhysicalKey::Code(KeyCode::PageDown) => E::PageDown,
        PhysicalKey::Code(KeyCode::ArrowLeft) => E::ArrowLeft,
        PhysicalKey::Code(KeyCode::ArrowUp) => E::ArrowUp,
        PhysicalKey::Code(KeyCode::ArrowRight) => E::ArrowRight,
        PhysicalKey::Code(KeyCode::ArrowDown) => E::ArrowDown,
        PhysicalKey::Code(KeyCode::Backspace) => E::Backspace,
        PhysicalKey::Code(KeyCode::Enter) => E::Enter,
        PhysicalKey::Code(KeyCode::Numpad0) => E::Num0,
        PhysicalKey::Code(KeyCode::Numpad1) => E::Num1,
        PhysicalKey::Code(KeyCode::Numpad2) => E::Num2,
        PhysicalKey::Code(KeyCode::Numpad3) => E::Num3,
        PhysicalKey::Code(KeyCode::Numpad4) => E::Num4,
        PhysicalKey::Code(KeyCode::Numpad5) => E::Num5,
        PhysicalKey::Code(KeyCode::Numpad6) => E::Num6,
        PhysicalKey::Code(KeyCode::Numpad7) => E::Num7,
        PhysicalKey::Code(KeyCode::Numpad8) => E::Num8,
        PhysicalKey::Code(KeyCode::Numpad9) => E::Num9,
        _ => return None,
    })
}

#[inline]
fn winit_to_egui_modifiers(modifiers: ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt_key(),
        ctrl: modifiers.control_key(),
        shift: modifiers.shift_key(),
        mac_cmd: false,
        command: modifiers.control_key(),
    }
}

/// We only want printable characters and ignore all special keys.
#[inline]
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}
