use eframe::egui;
use std::env;
use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;

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
            .with_inner_size([1200.0, 800.0])
            .with_title("TTY Doc - AI-Enhanced Text File Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "tty_doc",
        options,
        Box::new(move |_cc| Box::new(MyApp::new(file_path))),
    )
}

// Ollama API structures
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

// Chat message structure
#[derive(Clone, Debug)]
struct ChatMessage {
    role: String, // "user" or "assistant"
    content: String,
}

// AI state for managing async operations
#[derive(Clone)]
struct AiState {
    is_processing: Arc<Mutex<bool>>,
    current_response: Arc<Mutex<String>>,
    chat_history: Arc<Mutex<Vec<ChatMessage>>>,
    error: Arc<Mutex<Option<String>>>,
}

impl AiState {
    fn new() -> Self {
        Self {
            is_processing: Arc::new(Mutex::new(false)),
            current_response: Arc::new(Mutex::new(String::new())),
            chat_history: Arc::new(Mutex::new(Vec::new())),
            error: Arc::new(Mutex::new(None)),
        }
    }
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
    // AI fields
    ai_state: AiState,
    current_question: String,
    selected_model: String,
    available_models: Vec<String>,
    show_ai_panel: bool,
    ai_panel_width: f32,
    initial_summary_generated: bool,
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
            ai_state: AiState::new(),
            current_question: String::new(),
            selected_model: "llama2".to_string(),
            available_models: vec![
                "llama2".to_string(),
                "mistral".to_string(),
                "phi".to_string(),
                "codellama".to_string(),
            ],
            show_ai_panel: true,
            ai_panel_width: 400.0,
            initial_summary_generated: false,
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
                
                // Generate initial summary if content is loaded
                if !self.initial_summary_generated && !self.file_content.is_empty() {
                    self.generate_initial_summary();
                    self.initial_summary_generated = true;
                }
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

    fn generate_initial_summary(&mut self) {
        let truncated_content = if self.file_content.len() > 4000 {
            format!("{}...", &self.file_content[..4000])
        } else {
            self.file_content.clone()
        };

        let prompt = format!(
            "Please provide a concise summary of the following document. Focus on the main purpose, key points, and structure:\n\n{}\n\nSummary:",
            truncated_content
        );

        self.send_to_ai(prompt, true);
    }

    fn send_to_ai(&mut self, prompt: String, is_summary: bool) {
        let ai_state = self.ai_state.clone();
        let model = self.selected_model.clone();
        
        // Set processing state
        *ai_state.is_processing.lock().unwrap() = true;
        *ai_state.current_response.lock().unwrap() = String::new();
        *ai_state.error.lock().unwrap() = None;

        // Add user message to history (unless it's the initial summary)
        if !is_summary {
            ai_state.chat_history.lock().unwrap().push(ChatMessage {
                role: "user".to_string(),
                content: self.current_question.clone(),
            });
        }

        thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            
            let request = OllamaRequest {
                model,
                prompt,
                stream: false,
                options: OllamaOptions {
                    temperature: 0.7,
                    num_predict: 500,
                },
            };

            match client
                .post("http://localhost:11434/api/generate")
                .json(&request)
                .send()
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<OllamaResponse>() {
                            Ok(ollama_response) => {
                                *ai_state.current_response.lock().unwrap() = ollama_response.response.clone();
                                
                                // Add assistant response to history
                                ai_state.chat_history.lock().unwrap().push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: ollama_response.response,
                                });
                            }
                            Err(e) => {
                                *ai_state.error.lock().unwrap() = Some(format!("Failed to parse response: {}", e));
                            }
                        }
                    } else {
                        *ai_state.error.lock().unwrap() = Some(format!("API request failed: {}", response.status()));
                    }
                }
                Err(e) => {
                    *ai_state.error.lock().unwrap() = Some(format!("Failed to connect to Ollama. Make sure Ollama is running: {}", e));
                }
            }

            *ai_state.is_processing.lock().unwrap() = false;
        });
    }

    fn ask_question(&mut self) {
        if self.current_question.trim().is_empty() {
            return;
        }

        let truncated_content = if self.file_content.len() > 3000 {
            format!("{}...", &self.file_content[..3000])
        } else {
            self.file_content.clone()
        };

        let prompt = format!(
            "Based on the following document content:\n\n{}\n\nPlease answer this question: {}\n\nAnswer:",
            truncated_content,
            self.current_question
        );

        self.send_to_ai(prompt, false);
        self.current_question.clear();
    }

    fn clear_chat(&mut self) {
        self.ai_state.chat_history.lock().unwrap().clear();
        *self.ai_state.current_response.lock().unwrap() = String::new();
        *self.ai_state.error.lock().unwrap() = None;
        self.current_question.clear();
        
        // Regenerate initial summary
        if !self.file_content.is_empty() {
            self.generate_initial_summary();
        }
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
            ai_state: AiState::new(),
            current_question: String::new(),
            selected_model: "llama2".to_string(),
            available_models: vec![
                "llama2".to_string(),
                "mistral".to_string(),
                "phi".to_string(),
                "codellama".to_string(),
            ],
            show_ai_panel: true,
            ai_panel_width: 400.0,
            initial_summary_generated: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint if AI is processing
        if *self.ai_state.is_processing.lock().unwrap() {
            ctx.request_repaint();
        }

        // Top panel for controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Font Size:");
                ui.add(egui::Slider::new(&mut self.font_size, 8.0..=24.0).text("px"));
                
                ui.separator();
                
                if ui.button(if self.show_ai_panel { "Hide AI Panel" } else { "Show AI Panel" }).clicked() {
                    self.show_ai_panel = !self.show_ai_panel;
                }
                
                if self.show_ai_panel {
                    ui.separator();
                    ui.label("Model:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.selected_model)
                        .show_ui(ui, |ui| {
                            for model in &self.available_models {
                                ui.selectable_value(&mut self.selected_model, model.clone(), model);
                            }
                        });
                }
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
                
                if *self.ai_state.is_processing.lock().unwrap() {
                    ui.separator();
                    ui.label("🤖 AI is thinking...");
                }
            });
        });

        // AI Panel (Right side)
        if self.show_ai_panel {
            egui::SidePanel::right("ai_panel")
                .resizable(true)
                .default_width(self.ai_panel_width)
                .min_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("🤖 AI Assistant");
                    ui.separator();
                    
                    // Clear chat button
                    if ui.button("🗑 Clear Memory").clicked() {
                        self.clear_chat();
                    }
                    
                    ui.separator();
                    
                    // Chat history display
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height() - 100.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let chat_history = self.ai_state.chat_history.lock().unwrap();
                            
                            for message in chat_history.iter() {
                                ui.group(|ui| {
                                    if message.role == "user" {
                                        ui.label(egui::RichText::new("You:").strong());
                                    } else {
                                        ui.label(egui::RichText::new("AI:").strong().color(egui::Color32::from_rgb(100, 150, 255)));
                                    }
                                    ui.label(&message.content);
                                });
                                ui.add_space(5.0);
                            }
                            
                            // Show current processing response
                            if *self.ai_state.is_processing.lock().unwrap() {
                                ui.spinner();
                            }
                            
                            // Show error if any
                            if let Some(error) = &*self.ai_state.error.lock().unwrap() {
                                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                            }
                        });
                    
                    ui.separator();
                    
                    // Question input area
                    ui.horizontal(|ui| {
                        let response = ui.text_edit_singleline(&mut self.current_question);
                        
                        let is_processing = *self.ai_state.is_processing.lock().unwrap();
                        
                        if (ui.button("Send").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                            && !is_processing 
                        {
                            self.ask_question();
                        }
                    });
                    
                    ui.label(egui::RichText::new("Tip: Ask questions about the document content!").small());
                });
        }

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
                                    ui.spacing_mut().item_spacing.x = 0.0;
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