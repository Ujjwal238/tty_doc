use eframe::egui;
use std::env;
use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

fn main() -> Result<(), eframe::Error> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if file path argument is provided
    let file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        String::new()
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("TTY Doc - Text File Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "tty_doc",
        options,
        Box::new(move |_cc| Box::new(MyApp::new(file_path))),
    )
}

struct MyApp {
    file_path: String,
    file_content: String,
    error_message: Option<String>,
    font_size: f32,
    // Syntax highlighting fields
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    highlighted_content: Option<Vec<Vec<(Style, String)>>>,
}

impl MyApp {
    fn new(file_path: String) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        let mut app = Self {
            file_path: file_path.clone(),
            file_content: String::new(),
            error_message: None,
            font_size: 14.0,
            syntax_set,
            theme_set,
            highlighted_content: None,
        };

        if !file_path.is_empty() {
            app.load_file();
        } else {
            app.error_message = Some("No file path provided. Usage: tty_doc <file_path>".to_string());
        }
        
        app
    }

    fn detect_syntax(&self) -> Option<&syntect::parsing::SyntaxReference> {
        if let Some(extension) = Path::new(&self.file_path).extension() {
            if let Some(ext_str) = extension.to_str() {
                if let Some(syntax) = self.syntax_set.find_syntax_by_extension(ext_str) {
                    return Some(syntax);
                }
            }
        }
        
        // Fallback to filename detection
        self.syntax_set.find_syntax_for_file(&self.file_path).ok().flatten()
    }

    fn highlight_content(&mut self) {
        if self.file_content.is_empty() {
            self.highlighted_content = None;
            return;
        }

        let syntax = self.detect_syntax().unwrap_or_else(|| {
            self.syntax_set.find_syntax_plain_text()
        });

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);
        
        let mut highlighted_lines = Vec::new();
        
        for line in LinesWithEndings::from(&self.file_content) {
            let ranges = highlighter.highlight_line(line, &self.syntax_set)
                .unwrap_or_else(|_| vec![(Style::default(), line)]);
            // Convert &str to String for storage
            let string_ranges: Vec<(Style, String)> = ranges
                .into_iter()
                .map(|(style, text)| (style, text.to_string()))
                .collect();
            highlighted_lines.push(string_ranges);
        }
        
        self.highlighted_content = Some(highlighted_lines);
    }

    fn syntect_color_to_egui(color: syntect::highlighting::Color) -> egui::Color32 {
        egui::Color32::from_rgb(color.r, color.g, color.b)
    }

    fn load_file(&mut self) {
        if !Path::new(&self.file_path).exists() {
            self.error_message = Some(format!("File not found: {}", self.file_path));
            self.highlighted_content = None;
            return;
        }

        match fs::read_to_string(&self.file_path) {
            Ok(content) => {
                self.file_content = content;
                self.error_message = None;
                self.highlight_content();
            }
            Err(e) => {
                self.error_message = Some(format!("Error reading file: {}", e));
                self.highlighted_content = None;
            }
        }
    }

    fn get_file_info(&self) -> String {
        if self.file_content.is_empty() {
            return String::new();
        }
        
        let chars = self.file_content.chars().count();
        let bytes = self.file_content.len();
        let lines = self.file_content.lines().count();
        
        format!("Lines: {} | Characters: {} | Bytes: {}", lines, chars, bytes)
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        Self {
            file_path: String::new(),
            file_content: String::new(),
            error_message: Some("No file path provided. Usage: tty_doc <file_path>".to_string()),
            font_size: 14.0,
            syntax_set,
            theme_set,
            highlighted_content: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel for font size control
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Font Size:");
                ui.add(egui::Slider::new(&mut self.font_size, 8.0..=24.0).text("px"));
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if !self.file_path.is_empty() {
                    ui.label(format!("File: {}", self.file_path));
                    ui.separator();
                }
                ui.label(self.get_file_info());
            });
        });

        // Main content area
        let mut should_reload = false;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error_message {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.colored_label(egui::Color32::RED, "⚠ Error");
                    ui.label(error);
                    
                    if !self.file_path.is_empty() && ui.button("Try Again").clicked() {
                        should_reload = true;
                    }
                });
            } else {
                // Configure text style based on font size
                let mut style = ctx.style().as_ref().clone();
                style.text_styles.insert(
                    egui::TextStyle::Monospace,
                    egui::FontId::new(self.font_size, egui::FontFamily::Monospace),
                );
                ctx.set_style(style);

                // Display file content with syntax highlighting
                egui::ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        if let Some(highlighted_lines) = &self.highlighted_content {
                            // Display syntax highlighted content
                            for line_ranges in highlighted_lines {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0; // Remove spacing between text segments
                                    for (style, text) in line_ranges {
                                        let color = Self::syntect_color_to_egui(style.foreground);
                                        let rich_text = egui::RichText::new(text)
                                            .color(color)
                                            .font(egui::FontId::new(self.font_size, egui::FontFamily::Monospace));
                                        
                                        if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
                                            ui.add(egui::Label::new(rich_text.strong()));
                                        } else if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
                                            ui.add(egui::Label::new(rich_text.italics()));
                                        } else {
                                            ui.add(egui::Label::new(rich_text));
                                        }
                                    }
                                });
                            }
                        } else {
                            // Fallback to plain text
                            ui.add(
                                egui::TextEdit::multiline(&mut self.file_content.as_str())
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace)
                                    .interactive(false)
                            );
                        }
                    });
            }
        });
        
        // Handle file reload after UI to avoid borrowing conflicts
        if should_reload {
            self.load_file();
        }
    }
}