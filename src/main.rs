use eframe::egui;
use std::env;
use std::fs;
use std::path::Path;

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
}

impl MyApp {
    fn new(file_path: String) -> Self {
        let mut app = Self {
            file_path: file_path.clone(),
            file_content: String::new(),
            error_message: None,
            font_size: 14.0,
        };

        if !file_path.is_empty() {
            app.load_file();
        } else {
            app.error_message = Some("No file path provided. Usage: tty_doc <file_path>".to_string());
        }
        
        app
    }

    fn load_file(&mut self) {
        if !Path::new(&self.file_path).exists() {
            self.error_message = Some(format!("File not found: {}", self.file_path));
            return;
        }

        match fs::read_to_string(&self.file_path) {
            Ok(content) => {
                self.file_content = content;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Error reading file: {}", e));
            }
        }
    }

    fn get_file_info(&self) -> String {
        if self.file_content.is_empty() {
            return String::new();
        }
        
       
        let chars = self.file_content.chars().count();
        let bytes = self.file_content.len();
        
        format!("Characters: {} | Bytes: {}", chars, bytes)
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            file_content: String::new(),
            error_message: Some("No file path provided. Usage: tty_doc <file_path>".to_string()),
            font_size: 14.0,
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

                // Display file content
                egui::ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        let display_content = self.file_content.clone();
                        
                        ui.add(
                            egui::TextEdit::multiline(&mut display_content.as_str())
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false)
                        );
                    });
            }
        });
        
        // Handle file reload after UI to avoid borrowing conflicts
        if should_reload {
            self.load_file();
        }
    }
}