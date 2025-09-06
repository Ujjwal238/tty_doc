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
            .with_inner_size([640.0, 480.0]), // Made window larger for file content
        ..Default::default()
    };
    
    eframe::run_native(
        "File Viewer",
        options,
        Box::new(move |_cc| Box::new(MyApp::new(file_path))),
    )
}

struct MyApp {
    file_path: String,
    file_content: String,
    error_message: Option<String>,
}

impl MyApp {
    fn new(file_path: String) -> Self {
        let mut app = Self {
            file_path: file_path.clone(),
            file_content: String::new(),
            error_message: None,
        };
        
        if !file_path.is_empty() {
            app.load_file();
        } else {
            app.error_message = Some("No file path provided".to_string());
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
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            file_content: String::new(),
            error_message: Some("No file path provided".to_string()),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Viewer");
            ui.separator();
            
            // Display file path
            if !self.file_path.is_empty() {
                ui.label(format!("File: {}", self.file_path));
                ui.separator();
            }
            
            // Display error message if any
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                // Display file content in a scrollable area
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.file_content.as_str())
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace)
                        );
                    });
            }
        });
    }
}