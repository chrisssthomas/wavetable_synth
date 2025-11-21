use crate::voice::VoiceSettings;
use crate::oscillator::WaveType;
use crate::synth::PolySynth;
use crate::distortion::DistortionType;
use crate::reverb::ReverbType;

use eframe::egui::{self, Color32, Stroke, Pos2, Vec2};
use std::sync::Arc;
use std::collections::VecDeque;

// Professional synthesizer color scheme inspired by high-end VSTs
const BG_COLOR: Color32 = Color32::from_rgb(20, 22, 26);
const PANEL_COLOR: Color32 = Color32::from_rgb(28, 30, 34);
const ACCENT_COLOR: Color32 = Color32::from_rgb(0, 150, 255);
const KNOB_DARK: Color32 = Color32::from_rgb(45, 48, 52);
const TEXT_COLOR: Color32 = Color32::from_rgb(220, 220, 220);
const SCOPE_COLOR: Color32 = Color32::from_rgb(0, 255, 150);

pub struct SynthGui {
    synth: Arc<PolySynth>,
    settings: VoiceSettings,
    waveform_data: VecDeque<f32>,
    active_voices: usize,
}

impl SynthGui {
    pub fn new(synth: Arc<PolySynth>) -> Self {
        Self {
            synth,
            settings: VoiceSettings::default(),
            waveform_data: VecDeque::with_capacity(1024),
            active_voices: 0,
        }
    }

    fn apply_dark_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Dark theme colors
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(TEXT_COLOR);
        style.visuals.panel_fill = BG_COLOR;
        style.visuals.extreme_bg_color = BG_COLOR;
        style.visuals.faint_bg_color = PANEL_COLOR;
        style.visuals.window_fill = BG_COLOR;
        
        // Rounded corners for modern look
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.menu_rounding = egui::Rounding::same(6.0);
        
        ctx.set_style(style);
    }

    fn draw_oscilloscope(&self, ui: &mut egui::Ui, size: Vec2) {
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;
        
        // Background
        painter.rect_filled(rect, egui::Rounding::same(4.0), Color32::from_rgb(15, 17, 20));
        painter.rect_stroke(rect, egui::Rounding::same(4.0), Stroke::new(1.0, Color32::from_rgb(60, 65, 70)));
        
        // Grid lines (professional oscilloscope style)
        let grid_color = Color32::from_rgb(25, 30, 35);
        
        // Horizontal grid lines
        for i in 1..4 {
            let y = rect.min.y + (rect.height() / 4.0) * i as f32;
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(0.5, grid_color)
            );
        }
        
        // Vertical grid lines
        for i in 1..8 {
            let x = rect.min.x + (rect.width() / 8.0) * i as f32;
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(0.5, grid_color)
            );
        }
        
        // Center line (0V reference)
        let center_y = rect.center().y;
        painter.line_segment(
            [Pos2::new(rect.min.x, center_y), Pos2::new(rect.max.x, center_y)],
            Stroke::new(1.0, Color32::from_rgb(40, 45, 50))
        );
        
        // Draw waveform
        if self.waveform_data.len() > 1 {
            let mut points = Vec::new();
            let x_scale = rect.width() / (self.waveform_data.len() - 1) as f32;
            let y_center = rect.center().y;
            let y_scale = rect.height() * 0.4; // Use 80% of height for waveform
            
            for (i, &sample) in self.waveform_data.iter().enumerate() {
                let x = rect.min.x + i as f32 * x_scale;
                let y = y_center - sample * y_scale;
                points.push(Pos2::new(x, y));
            }
            
            if points.len() > 1 {
                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(1.5, SCOPE_COLOR)
                ));
            }
        }
        
        // Scope label
        painter.text(
            rect.min + Vec2::new(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            "OSCILLOSCOPE",
            egui::FontId::monospace(10.0),
            Color32::from_rgb(140, 145, 150)
        );
    }

    // Professional VST-style knob inspired by Serum, Massive, etc.
    fn draw_vst_knob(
        ui: &mut egui::Ui,
        label: &str,
        value: &mut f32,
        min: f32,
        max: f32,
        size: f32,
        unit: &str,
    ) -> bool {
        let mut changed = false;
        
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            
            // Label
            ui.label(egui::RichText::new(label).size(11.0).color(TEXT_COLOR));
            
            let (response, painter) = ui.allocate_painter(
                Vec2::splat(size),
                egui::Sense::click_and_drag()
            );
            
            let rect = response.rect;
            let center = rect.center();
            let radius = size * 0.4;
            
            // Calculate angle (-140° to +140°, like real synth knobs)
            let normalized = (*value - min) / (max - min);
            let angle = -2.44 + normalized * 4.88; // 280 degrees total range
            
            // Drop shadow for depth
            painter.circle_filled(
                center + Vec2::new(1.5, 1.5),
                radius + 1.0,
                Color32::from_rgba_premultiplied(0, 0, 0, 60)
            );
            
            // Outer ring (chrome/metal bezel like real hardware)
            let gradient_stops = 8;
            for i in 0..gradient_stops {
                let ring_radius = radius + 2.0 - (i as f32 * 0.3);
                let brightness = 90 + (i * 8);
                painter.circle_stroke(
                    center,
                    ring_radius,
                    Stroke::new(0.5, Color32::from_rgb(brightness, brightness + 5, brightness + 10))
                );
            }
            
            // Main knob body (matte black like high-end synths)
            painter.circle_filled(center, radius, KNOB_DARK);
            
            // Inner highlight (subtle)
            painter.circle_filled(
                center - Vec2::new(radius * 0.2, radius * 0.3),
                radius * 0.3,
                Color32::from_rgba_premultiplied(80, 85, 90, 30)
            );
            
            // Value arc (like Serum/Massive knobs)
            let arc_radius = radius - 4.0;
            let arc_thickness = 2.5;
            let arc_segments = (normalized * 60.0) as i32; // 60 segments for smooth arc
            
            for i in 0..arc_segments {
                let seg_angle = -2.44 + (i as f32 / 60.0) * 4.88;
                let seg_pos = center + Vec2::new(
                    seg_angle.cos() * arc_radius,
                    seg_angle.sin() * arc_radius
                );
                
                // Color gradient from blue to cyan
                let seg_color = if i < 30 {
                    Color32::from_rgb(0, 100 + (i as u8 * 3), 255)
                } else {
                    Color32::from_rgb(0, 190, 255 - ((i - 30) as u8 * 2))
                };
                
                painter.circle_filled(seg_pos, arc_thickness * 0.6, seg_color);
            }
            
            // Position indicator (white dot like professional knobs)
            let indicator_pos = center + Vec2::new(
                angle.cos() * (radius - 8.0),
                angle.sin() * (radius - 8.0)
            );
            
            painter.circle_filled(indicator_pos, 2.5, Color32::WHITE);
            painter.circle_stroke(indicator_pos, 2.5, Stroke::new(0.8, KNOB_DARK));
            
            // Mouse interaction
            if response.dragged() {
                let drag_delta = response.drag_delta();
                let sensitivity = 0.006;
                *value += drag_delta.y * -(max - min) * sensitivity;
                *value = value.clamp(min, max);
                changed = true;
            }
            
            // Value display (professional formatting)
            let value_text = if unit.is_empty() {
                if *value < 1.0 {
                    format!("{:.3}", value)
                } else if *value < 10.0 {
                    format!("{:.2}", value)
                } else {
                    format!("{:.1}", value)
                }
            } else if unit == "Hz" && *value >= 1000.0 {
                format!("{:.1}k", *value / 1000.0)
            } else {
                format!("{:.1}{}", value, unit)
            };
            
            ui.label(
                egui::RichText::new(&value_text)
                    .size(9.5)
                    .color(ACCENT_COLOR)
                    .family(egui::FontFamily::Monospace)
            );
        });
        
        changed
    }

    fn draw_waveform_selector(ui: &mut egui::Ui, wave_type: &mut WaveType, label: &str) -> bool {
        let mut changed = false;
        
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).size(11.0).color(TEXT_COLOR));
            
            ui.horizontal(|ui| {
                let options = [
                    (WaveType::Sine, "SIN"),
                    (WaveType::Sawtooth, "SAW"),
                    (WaveType::Square, "SQR"),
                    (WaveType::Triangle, "TRI"),
                ];
                
                for (wt, name) in options {
                    let is_selected = *wave_type == wt;
                    let button_color = if is_selected { ACCENT_COLOR } else { KNOB_DARK };
                    let text_color = if is_selected { Color32::WHITE } else { TEXT_COLOR };
                    
                    if ui.add(
                        egui::Button::new(egui::RichText::new(name).size(9.0).color(text_color))
                            .fill(button_color)
                            .min_size(Vec2::new(32.0, 22.0))
                            .rounding(egui::Rounding::same(3.0))
                    ).clicked() {
                        *wave_type = wt;
                        changed = true;
                    }
                }
            });
        });
        
        changed
    }
}

impl eframe::App for SynthGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_dark_style(ctx);
        
        // Update waveform data from synthesizer
        if let Some(buffer) = self.synth.get_waveform_buffer() {
            self.waveform_data = buffer;
        }
        
        // Update active voices count
        self.active_voices = self.synth.get_active_voice_count();

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(BG_COLOR).inner_margin(16.0))
            .show(ctx, |ui| {
                
                // === RESPONSIVE LAYOUT WITH SCROLLING ===
                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        
                        // === CLEAN HEADER ===
                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("ANALOG SYNTHESIZER")
                                    .size(16.0)
                                    .color(TEXT_COLOR)
                                    .strong()
                            );
                            ui.add_space(16.0);
                        });
                
                        // === TOP ROW: OSCILLATORS ===
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 16.0;
                            
                            // OSC 1 Panel
                            egui::Frame::none()
                                .fill(PANEL_COLOR)
                                .rounding(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(180.0);
                                    ui.vertical(|ui| {
                                        // Section header
                                        ui.label(
                                            egui::RichText::new("OSCILLATOR 1")
                                                .size(13.0)
                                                .color(ACCENT_COLOR)
                                                .strong()
                                        );
                                        ui.add_space(12.0);
                                        
                                        // Waveform controls
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 12.0;
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_waveform_selector(ui, &mut self.settings.osc1_wave_type, "WAVE") {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "SHAPE",
                                                    &mut self.settings.osc1_shape,
                                                    0.0,
                                                    1.0,
                                                    45.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                            
                            // OSC 2 Panel
                            egui::Frame::none()
                                .fill(PANEL_COLOR)
                                .rounding(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(180.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("OSCILLATOR 2")
                                                .size(13.0)
                                                .color(ACCENT_COLOR)
                                                .strong()
                                        );
                                        ui.add_space(12.0);
                                        
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 12.0;
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_waveform_selector(ui, &mut self.settings.osc2_wave_type, "WAVE") {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "SHAPE",
                                                    &mut self.settings.osc2_shape,
                                                    0.0,
                                                    1.0,
                                                    45.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                            
                            // MIX Panel
                            egui::Frame::none()
                                .fill(PANEL_COLOR)
                                .rounding(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(140.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("MIX")
                                                .size(13.0)
                                                .color(ACCENT_COLOR)
                                                .strong()
                                        );
                                        ui.add_space(12.0);
                                        
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 12.0;
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "BLEND",
                                                    &mut self.settings.osc_mix,
                                                    0.0,
                                                    1.0,
                                                    45.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "DETUNE",
                                                    &mut self.settings.osc2_detune,
                                                    -50.0,
                                                    50.0,
                                                    45.0,
                                                    "¢"
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                        });
                        
                        ui.add_space(16.0);                // === MIDDLE ROW: FILTER & ENVELOPE ===
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 32.0;
                    
                    // Filter Panel
                    egui::Frame::none()
                        .fill(PANEL_COLOR)
                        .rounding(8.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.set_min_width(200.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("FILTER")
                                        .size(13.0)
                                        .color(ACCENT_COLOR)
                                        .strong()
                                );
                                ui.add_space(16.0);
                                
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 16.0;
                                    
                                    ui.vertical(|ui| {
                                        if Self::draw_vst_knob(
                                            ui,
                                            "FREQ",
                                            &mut self.settings.filter_freq,
                                            200.0,
                                            8000.0,
                                            50.0,
                                            "Hz"
                                        ) {
                                            self.synth.update_settings(self.settings.clone());
                                        }
                                    });
                                    
                                    ui.vertical(|ui| {
                                        if Self::draw_vst_knob(
                                            ui,
                                            "RES",
                                            &mut self.settings.filter_resonance,
                                            0.0,
                                            0.95,
                                            50.0,
                                            ""
                                        ) {
                                            self.synth.update_settings(self.settings.clone());
                                        }
                                    });
                                });
                            });
                        });
                    
                    // Envelope Panel
                    egui::Frame::none()
                        .fill(PANEL_COLOR)
                        .rounding(8.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.set_min_width(320.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("ENVELOPE")
                                        .size(13.0)
                                        .color(ACCENT_COLOR)
                                        .strong()
                                );
                                ui.add_space(16.0);
                                
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 16.0;
                                    
                                    ui.vertical(|ui| {
                                        if Self::draw_vst_knob(
                                            ui,
                                            "ATTACK",
                                            &mut self.settings.attack_time,
                                            0.001,
                                            2.0,
                                            50.0,
                                            "s"
                                        ) {
                                            self.synth.update_settings(self.settings.clone());
                                        }
                                    });
                                    
                                    ui.vertical(|ui| {
                                        if Self::draw_vst_knob(
                                            ui,
                                            "DECAY",
                                            &mut self.settings.decay_time,
                                            0.001,
                                            3.0,
                                            50.0,
                                            "s"
                                        ) {
                                            self.synth.update_settings(self.settings.clone());
                                        }
                                    });
                                    
                                    ui.vertical(|ui| {
                                        if Self::draw_vst_knob(
                                            ui,
                                            "SUSTAIN",
                                            &mut self.settings.sustain_level,
                                            0.0,
                                            1.0,
                                            50.0,
                                            ""
                                        ) {
                                            self.synth.update_settings(self.settings.clone());
                                        }
                                    });
                                    
                                    ui.vertical(|ui| {
                                        if Self::draw_vst_knob(
                                            ui,
                                            "RELEASE",
                                            &mut self.settings.release_time,
                                            0.001,
                                            5.0,
                                            50.0,
                                            "s"
                                        ) {
                                            self.synth.update_settings(self.settings.clone());
                                        }
                                    });
                                });
                            });
                        });
                });
                
                        ui.add_space(16.0);
                        
                        // === BOTTOM ROW: EFFECTS ===
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 16.0;
                            
                            // Distortion Panel
                            egui::Frame::none()
                                .fill(PANEL_COLOR)
                                .rounding(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(220.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("DISTORTION")
                                                .size(13.0)
                                                .color(ACCENT_COLOR)
                                                .strong()
                                        );
                                        ui.add_space(12.0);
                                
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 12.0;
                                            
                                            // Distortion type selector
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new("TYPE").size(10.0).color(TEXT_COLOR));
                                                ui.add_space(4.0);
                                                egui::ComboBox::from_id_source("distortion_type")
                                                    .selected_text(egui::RichText::new(format!("{:?}", self.settings.distortion_type)).color(TEXT_COLOR))
                                                    .width(70.0)
                                                    .show_ui(ui, |ui| {
                                                        ui.style_mut().visuals.widgets.inactive.fg_stroke.color = TEXT_COLOR;
                                                        ui.style_mut().visuals.widgets.hovered.fg_stroke.color = TEXT_COLOR;
                                                        ui.style_mut().visuals.widgets.active.fg_stroke.color = TEXT_COLOR;
                                                        ui.style_mut().visuals.widgets.inactive.bg_fill = KNOB_DARK;
                                                        ui.style_mut().visuals.widgets.hovered.bg_fill = PANEL_COLOR;
                                                        ui.style_mut().visuals.widgets.active.bg_fill = ACCENT_COLOR;
                                                        
                                                        let types = [
                                                            DistortionType::Clean,
                                                            DistortionType::Overdrive,
                                                            DistortionType::Distortion,
                                                            DistortionType::Fuzz,
                                                            DistortionType::Tube,
                                                        ];
                                                        for dist_type in types.iter() {
                                                            let response = ui.selectable_value(&mut self.settings.distortion_type, *dist_type, 
                                                                egui::RichText::new(format!("{:?}", dist_type)).color(TEXT_COLOR));
                                                            if response.clicked() {
                                                                self.synth.update_settings(self.settings.clone());
                                                            }
                                                        }
                                                    });
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "DRIVE",
                                                    &mut self.settings.distortion_drive,
                                                    0.0,
                                                    1.0,
                                                    40.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "TONE",
                                                    &mut self.settings.distortion_tone,
                                                    0.0,
                                                    1.0,
                                                    40.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "LEVEL",
                                                    &mut self.settings.distortion_level,
                                                    0.0,
                                                    1.0,
                                                    40.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                            
                            // Reverb Panel
                            egui::Frame::none()
                                .fill(PANEL_COLOR)
                                .rounding(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(220.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("REVERB")
                                                .size(13.0)
                                                .color(ACCENT_COLOR)
                                                .strong()
                                        );
                                        ui.add_space(12.0);
                                        
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 12.0;
                                            
                                            // Reverb type selector
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new("TYPE").size(10.0).color(TEXT_COLOR));
                                                ui.add_space(4.0);
                                                egui::ComboBox::from_id_source("reverb_type")
                                                    .selected_text(egui::RichText::new(format!("{:?}", self.settings.reverb_type)).color(TEXT_COLOR))
                                                    .width(70.0)
                                                    .show_ui(ui, |ui| {
                                                        ui.style_mut().visuals.widgets.inactive.fg_stroke.color = TEXT_COLOR;
                                                        ui.style_mut().visuals.widgets.hovered.fg_stroke.color = TEXT_COLOR;
                                                        ui.style_mut().visuals.widgets.active.fg_stroke.color = TEXT_COLOR;
                                                        ui.style_mut().visuals.widgets.inactive.bg_fill = KNOB_DARK;
                                                        ui.style_mut().visuals.widgets.hovered.bg_fill = PANEL_COLOR;
                                                        ui.style_mut().visuals.widgets.active.bg_fill = ACCENT_COLOR;
                                                        
                                                        let types = [
                                                            ReverbType::Room,
                                                            ReverbType::Hall,
                                                            ReverbType::Plate,
                                                            ReverbType::Spring,
                                                        ];
                                                        for reverb_type in types.iter() {
                                                            let response = ui.selectable_value(&mut self.settings.reverb_type, *reverb_type, 
                                                                egui::RichText::new(format!("{:?}", reverb_type)).color(TEXT_COLOR));
                                                            if response.clicked() {
                                                                self.synth.update_settings(self.settings.clone());
                                                            }
                                                        }
                                                    });
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "SIZE",
                                                    &mut self.settings.reverb_size,
                                                    0.0,
                                                    1.0,
                                                    40.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "DECAY",
                                                    &mut self.settings.reverb_decay,
                                                    0.1,
                                                    0.99,
                                                    40.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                            
                                            ui.vertical(|ui| {
                                                if Self::draw_vst_knob(
                                                    ui,
                                                    "MIX",
                                                    &mut self.settings.reverb_mix,
                                                    0.0,
                                                    1.0,
                                                    40.0,
                                                    ""
                                                ) {
                                                    self.synth.update_settings(self.settings.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                        });
                        
                        ui.add_space(16.0);
                        
                        // === OSCILLOSCOPE & MASTER ===
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 16.0;
                            
                            // Oscilloscope Panel
                            egui::Frame::none()
                                .fill(PANEL_COLOR)
                                .rounding(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(320.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("OSCILLOSCOPE")
                                                .size(13.0)
                                                .color(ACCENT_COLOR)
                                                .strong()
                                        );
                                        ui.add_space(12.0);
                                        
                                        self.draw_oscilloscope(ui, Vec2::new(300.0, 100.0));
                                    });
                                });
                            
                            // Master Panel
                            egui::Frame::none()
                                .fill(PANEL_COLOR)
                                .rounding(8.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(120.0);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("MASTER")
                                                .size(13.0)
                                                .color(ACCENT_COLOR)
                                                .strong()
                                        );
                                        ui.add_space(12.0);
                                        
                                        ui.vertical_centered(|ui| {
                                            if Self::draw_vst_knob(
                                                ui,
                                                "VOLUME",
                                                &mut self.settings.master_volume,
                                                0.0,
                                                2.0,
                                                50.0,
                                                ""
                                            ) {
                                                self.synth.update_settings(self.settings.clone());
                                            }
                                            
                                            ui.add_space(12.0);
                                            
                                            ui.label(
                                                egui::RichText::new(&format!("Active Voices: {}/16", self.active_voices))
                                                    .size(11.0)
                                                    .color(ACCENT_COLOR)
                                            );
                                        });
                                    });
                                });
                        });
                        
                    }); // End of scroll area
                
                ui.add_space(16.0);
                
                // Clean footer
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Connect MIDI device and press keys to play")
                            .size(10.0)
                            .color(Color32::from_rgb(140, 145, 150))
                    );
                });
            });
        
        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}