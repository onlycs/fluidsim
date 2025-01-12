// some stuff taken from https://github.com/NemuiSen/ggegui

use std::{
    collections::{HashMap, LinkedList},
    ops::Deref,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use egui::*;
use epaint::Primitive;
use ggez::{
    context::Has,
    graphics::{
        self, BlendComponent, BlendFactor, BlendMode, BlendOperation, Canvas, DrawParam, Drawable,
        GraphicsContext, ImageFormat,
    },
    mint::Point2,
    winit::event::MouseButton,
};

#[cfg(not(target_arch = "wasm32"))]
use ggez::{
    input::keyboard,
    winit::keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

#[cfg(target_arch = "wasm32")]
use ggez::input::keyboard::{KeyCode, KeyMods};

use itertools::Itertools;

pub struct PaintJob {
    texture: TextureId,
    mesh: graphics::Mesh,
    rect: graphics::Rect,
}

pub struct GuiContext<'a> {
    gui: &'a mut EguiTranslator,
}

impl<'a> Deref for GuiContext<'a> {
    type Target = egui::Context;

    fn deref(&self) -> &Self::Target {
        &self.gui.ctx
    }
}

impl<'a> Drop for GuiContext<'a> {
    fn drop(&mut self) {
        let FullOutput {
            shapes,
            pixels_per_point,
            textures_delta,
            ..
        } = self.end_pass();

        self.gui.shapes = self.gui.ctx.tessellate(shapes, pixels_per_point);
        self.gui.textures_delta.push_front(textures_delta);
    }
}

pub struct Input {
    dt: Instant,
    pointer: Pos2,
    raw: RawInput,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            dt: Instant::now(),
            pointer: Default::default(),
            raw: Default::default(),
        }
    }
}

impl Input {
    fn take(&mut self) -> RawInput {
        self.raw.predicted_dt = self.dt.elapsed().as_secs_f32();
        self.dt = Instant::now();

        self.raw.take()
    }

    fn update(&mut self, ctx: &mut ggez::Context) {
        let egui_mods;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let ggez_mods = ctx.keyboard.active_modifiers;
            egui_mods = Modifiers {
                alt: ggez_mods.intersects(ModifiersState::ALT),
                ctrl: ggez_mods.intersects(ModifiersState::CONTROL),
                shift: ggez_mods.intersects(ModifiersState::SHIFT),
                mac_cmd: cfg!(target_os = "macos") && ggez_mods.intersects(ModifiersState::SUPER),
                command: if cfg!(target_os = "macos") {
                    ggez_mods.intersects(ModifiersState::SUPER)
                } else {
                    ggez_mods.intersects(ModifiersState::CONTROL)
                },
            };

            for key in &ctx.keyboard.pressed_physical_keys {
                let PhysicalKey::Code(kc) = key else {
                    continue;
                };

                let Some(egui_key) = translate_key(kc) else {
                    continue;
                };

                if ctx.keyboard.is_physical_key_just_pressed(&key) || ctx.keyboard.is_key_repeated()
                {
                    self.raw.events.push(Event::Key {
                        key: egui_key,
                        physical_key: None,
                        pressed: true,
                        repeat: ctx.keyboard.is_key_repeated(),
                        modifiers: egui_mods,
                    });
                }

                self.raw.events.push(Event::Text(
                    ctx.keyboard
                        .pressed_logical_keys
                        .iter()
                        .filter(|k| {
                            (ctx.keyboard.is_logical_key_just_pressed(k))
                                && ctx.keyboard.active_modifiers.is_empty()
                        })
                        .filter_map(|k| match k {
                            keyboard::Key::Character(ch) => Some(ch.as_str()),
                            _ => None,
                        })
                        .collect(),
                ));
            }
        };

        #[cfg(target_arch = "wasm32")]
        {
            let ggez_mods = ctx.keyboard.active_mods();
            egui_mods = Modifiers {
                alt: ggez_mods.intersects(KeyMods::ALT),
                ctrl: ggez_mods.intersects(KeyMods::CTRL),
                shift: ggez_mods.intersects(KeyMods::SHIFT),
                mac_cmd: false,
                command: false,
            };

            for key in ctx.keyboard.pressed_keys() {
                let Some(egui_key) = translate_key(key) else {
                    continue;
                };

                if ctx.keyboard.is_key_just_pressed(*key) || ctx.keyboard.is_key_repeated() {
                    self.raw.events.push(Event::Key {
                        key: egui_key,
                        physical_key: None,
                        pressed: true,
                        repeat: ctx.keyboard.is_key_repeated(),
                        modifiers: egui_mods,
                    });
                }

                self.raw.events.push(Event::Text(
                    ctx.keyboard
                        .pressed_keys()
                        .iter()
                        .filter(|k| {
                            (ctx.keyboard.is_key_just_pressed(**k))
                                && ctx.keyboard.active_mods().is_empty()
                        })
                        .filter_map(|k| translate_key(k))
                        .map(|c| c.symbol_or_name())
                        .collect(),
                ));
            }
        }

        let Point2 { x, y } = ctx.mouse.position();
        self.pointer = Pos2::new(x, y);
        self.raw.events.push(Event::PointerMoved(self.pointer));

        for btn in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
            if ctx.mouse.button_just_pressed(btn) {
                self.raw.events.push(Event::PointerButton {
                    pos: self.pointer,
                    button: match btn {
                        MouseButton::Left => PointerButton::Primary,
                        MouseButton::Middle => PointerButton::Middle,
                        MouseButton::Right => PointerButton::Secondary,
                        _ => unreachable!(),
                    },
                    pressed: true,
                    modifiers: egui_mods,
                })
            }

            if ctx.mouse.button_just_released(btn) {
                self.raw.events.push(Event::PointerButton {
                    pos: self.pointer,
                    button: match btn {
                        MouseButton::Left => PointerButton::Primary,
                        MouseButton::Middle => PointerButton::Middle,
                        MouseButton::Right => PointerButton::Secondary,
                        _ => unreachable!(),
                    },
                    pressed: false,
                    modifiers: egui_mods,
                })
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn translate_key(kc: &KeyCode) -> Option<Key> {
    Some(match kc {
        // Letter keys
        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,

        // Punctuation et. al
        KeyCode::BracketLeft => Key::OpenBracket,
        KeyCode::BracketRight => Key::CloseBracket,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Quote => Key::Quote,
        KeyCode::Comma => Key::Comma,
        KeyCode::Period => Key::Period,
        KeyCode::Slash => Key::Slash,
        KeyCode::Space => Key::Space,

        // Function Row
        KeyCode::Escape => Key::Escape,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,
        KeyCode::F21 => Key::F21,
        KeyCode::F22 => Key::F22,
        KeyCode::F23 => Key::F23,
        KeyCode::F24 => Key::F24,

        // Numeral Row
        KeyCode::Backquote => Key::Backtick,
        KeyCode::Digit1 => Key::Num1,
        KeyCode::Digit2 => Key::Num2,
        KeyCode::Digit3 => Key::Num3,
        KeyCode::Digit4 => Key::Num4,
        KeyCode::Digit5 => Key::Num5,
        KeyCode::Digit6 => Key::Num6,
        KeyCode::Digit7 => Key::Num7,
        KeyCode::Digit8 => Key::Num8,
        KeyCode::Digit9 => Key::Num9,
        KeyCode::Digit0 => Key::Num0,

        // Numeral Row Math
        KeyCode::Minus => Key::Minus,
        KeyCode::Equal => Key::Equals,

        // Text Input Control
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Insert => Key::Insert,
        KeyCode::Home => Key::Home,
        KeyCode::Delete => Key::Delete,
        KeyCode::End => Key::End,
        KeyCode::Enter => Key::Enter,

        // Arrow Keys et. al
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowUp => Key::ArrowUp,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::PageUp => Key::PageUp,

        // Numpad keys
        KeyCode::Numpad0 => Key::Num0,
        KeyCode::Numpad1 => Key::Num1,
        KeyCode::Numpad2 => Key::Num2,
        KeyCode::Numpad3 => Key::Num3,
        KeyCode::Numpad4 => Key::Num4,
        KeyCode::Numpad5 => Key::Num5,
        KeyCode::Numpad6 => Key::Num6,
        KeyCode::Numpad7 => Key::Num7,
        KeyCode::Numpad8 => Key::Num8,
        KeyCode::Numpad9 => Key::Num9,

        // Close enough
        _ => return None,
    })
}

#[cfg(target_arch = "wasm32")]
fn translate_key(kc: &KeyCode) -> Option<Key> {
    Some(match kc {
        // Letter keys
        KeyCode::A => Key::A,
        KeyCode::B => Key::B,
        KeyCode::C => Key::C,
        KeyCode::D => Key::D,
        KeyCode::E => Key::E,
        KeyCode::F => Key::F,
        KeyCode::G => Key::G,
        KeyCode::H => Key::H,
        KeyCode::I => Key::I,
        KeyCode::J => Key::J,
        KeyCode::K => Key::K,
        KeyCode::L => Key::L,
        KeyCode::M => Key::M,
        KeyCode::N => Key::N,
        KeyCode::O => Key::O,
        KeyCode::P => Key::P,
        KeyCode::Q => Key::Q,
        KeyCode::R => Key::R,
        KeyCode::S => Key::S,
        KeyCode::T => Key::T,
        KeyCode::U => Key::U,
        KeyCode::V => Key::V,
        KeyCode::W => Key::W,
        KeyCode::X => Key::X,
        KeyCode::Y => Key::Y,
        KeyCode::Z => Key::Z,

        // Punctuation et. al
        KeyCode::LBracket => Key::OpenBracket,
        KeyCode::RBracket => Key::CloseBracket,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Apostrophe => Key::Quote,
        KeyCode::Comma => Key::Comma,
        KeyCode::Period => Key::Period,
        KeyCode::Slash => Key::Slash,
        KeyCode::Space => Key::Space,

        // Function Row
        KeyCode::Escape => Key::Escape,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,
        KeyCode::F21 => Key::F21,
        KeyCode::F22 => Key::F22,
        KeyCode::F23 => Key::F23,
        KeyCode::F24 => Key::F24,

        // Numeral Row
        KeyCode::Grave => Key::Backtick,
        KeyCode::Key1 => Key::Num1,
        KeyCode::Key2 => Key::Num2,
        KeyCode::Key3 => Key::Num3,
        KeyCode::Key4 => Key::Num4,
        KeyCode::Key5 => Key::Num5,
        KeyCode::Key6 => Key::Num6,
        KeyCode::Key7 => Key::Num7,
        KeyCode::Key8 => Key::Num8,
        KeyCode::Key9 => Key::Num9,
        KeyCode::Key0 => Key::Num0,

        // Numeral Row Math
        KeyCode::Minus => Key::Minus,
        KeyCode::Equals => Key::Equals,

        // Text Input Control
        KeyCode::Back => Key::Backspace,
        KeyCode::Insert => Key::Insert,
        KeyCode::Home => Key::Home,
        KeyCode::Delete => Key::Delete,
        KeyCode::End => Key::End,
        KeyCode::Return => Key::Enter,

        // Arrow Keys et. al
        KeyCode::Left => Key::ArrowLeft,
        KeyCode::Up => Key::ArrowUp,
        KeyCode::Right => Key::ArrowRight,
        KeyCode::Down => Key::ArrowDown,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::PageUp => Key::PageUp,

        // Numpad keys
        KeyCode::Numpad0 => Key::Num0,
        KeyCode::Numpad1 => Key::Num1,
        KeyCode::Numpad2 => Key::Num2,
        KeyCode::Numpad3 => Key::Num3,
        KeyCode::Numpad4 => Key::Num4,
        KeyCode::Numpad5 => Key::Num5,
        KeyCode::Numpad6 => Key::Num6,
        KeyCode::Numpad7 => Key::Num7,
        KeyCode::Numpad8 => Key::Num8,
        KeyCode::Numpad9 => Key::Num9,

        // Close enough
        _ => return None,
    })
}

pub struct EguiTranslator {
    ctx: egui::Context,
    shapes: Vec<ClippedPrimitive>,
    paints: Vec<PaintJob>,
    textures: HashMap<TextureId, graphics::Image>,
    textures_delta: LinkedList<TexturesDelta>,
    input: Input,
    on: bool,
}

impl Default for EguiTranslator {
    fn default() -> Self {
        Self {
            ctx: Default::default(),
            shapes: Default::default(),
            paints: Default::default(),
            textures: Default::default(),
            textures_delta: Default::default(),
            input: Default::default(),
            on: true,
        }
    }
}

impl EguiTranslator {
    pub fn toggle(&mut self) {
        self.on = !self.on;
    }

    pub fn update(&mut self, ctx: &mut ggez::Context) {
        if !self.on {
            return;
        }

        self.input.update(ctx);

        while let Some(tex) = self.textures_delta.pop_front() {
            for (id, delta) in tex.set {
                if delta.pos.is_some() {
                    error!("Textures with nonzero offsets not implemented");
                    continue;
                }

                let image = match delta.image {
                    ImageData::Color(col) => {
                        let mut pixels = Vec::with_capacity(col.pixels.len() * 4);

                        for px in &col.pixels {
                            pixels.extend(px.to_array());
                        }
                        graphics::Image::from_pixels(
                            ctx,
                            pixels.as_slice(),
                            ImageFormat::Rgba8UnormSrgb,
                            col.width() as u32,
                            col.height() as u32,
                        )
                    }
                    ImageData::Font(font) => {
                        let mut pixels = Vec::with_capacity(font.pixels.len() * 4);

                        for px in font.srgba_pixels(None) {
                            pixels.extend(px.to_array());
                        }

                        graphics::Image::from_pixels(
                            ctx,
                            pixels.as_slice(),
                            ImageFormat::Rgba8UnormSrgb,
                            font.width() as u32,
                            font.height() as u32,
                        )
                    }
                };

                self.textures.insert(id, image);
            }

            for id in &tex.free {
                self.textures.remove(id);
            }
        }

        for egui::ClippedPrimitive {
            clip_rect,
            primitive,
        } in self.shapes.iter()
        {
            match primitive {
                Primitive::Mesh(mesh) => {
                    if mesh.vertices.len() < 3 {
                        continue;
                    }

                    let verts = mesh.vertices.iter().map(|v| graphics::Vertex {
                        position: [v.pos.x, v.pos.y],
                        uv: [v.uv.x, v.uv.y],
                        color: Rgba::from(v.color).to_array(),
                    });

                    self.paints.push(PaintJob {
                        texture: mesh.texture_id,
                        mesh: graphics::Mesh::from_data(
                            ctx,
                            graphics::MeshData {
                                vertices: verts.collect_vec().as_slice(),
                                indices: mesh.indices.as_slice(),
                            },
                        ),
                        rect: graphics::Rect {
                            x: clip_rect.min.x,
                            y: clip_rect.min.y,
                            w: clip_rect.width(),
                            h: clip_rect.height(),
                        },
                    })
                }
                Primitive::Callback(_) => panic!("Callback not implemented"),
            }
        }
    }

    pub fn ctx<'a>(&'a mut self) -> GuiContext<'a> {
        self.paints.clear();
        self.ctx.begin_pass(self.input.take());
        GuiContext { gui: self }
    }
}

impl Drawable for EguiTranslator {
    fn draw(&self, canvas: &mut Canvas, param: impl Into<DrawParam>) {
        let prev_blend = canvas.blend_mode();
        let param: DrawParam = param.into();

        canvas.set_blend_mode(BlendMode {
            color: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::OneMinusDstAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
        });
        for PaintJob {
            texture,
            mesh,
            rect,
        } in self.paints.iter()
        {
            canvas.set_scissor_rect(*rect).unwrap();
            canvas.draw_textured_mesh(mesh.clone(), self.textures[texture].clone(), param);
        }
        canvas.set_default_scissor_rect();
        canvas.set_blend_mode(prev_blend);
    }

    fn dimensions(&self, gfx: &impl Has<graphics::GraphicsContext>) -> graphics::Rect {
        let gfx: &GraphicsContext = gfx.retrieve();
        let (w, h) = gfx.size();

        graphics::Rect { x: 0., y: 0., w, h }
    }
}
