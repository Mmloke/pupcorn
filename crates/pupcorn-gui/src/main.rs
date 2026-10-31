use eframe::egui;
use pupcorn_core::default_engine;
use pupcorn_ai::{Agent, OpenAiClient};
use std::sync::mpsc;

struct PupcornApp {
    goal: String,
    log: String,
    tools: Vec<String>,
    rx: Option<mpsc::Receiver<String>>,
    running: bool,
    selected_tab: usize,
}

impl PupcornApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let engine = default_engine();
        let tools = engine.list().iter().map(|(n, d, i)| format!("{}{} — {}", if *i { "🔒 " } else { "🟢 " }, n, d)).collect();
        Self {
            goal: String::new(),
            log: String::new(),
            tools,
            rx: None,
            running: false,
            selected_tab: 0,
        }
    }
}

impl eframe::App for PupcornApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain results
        if let Some(rx) = &self.rx {
            let mut done = false;
            while let Ok(msg) = rx.try_recv() {
                if msg == "__DONE__" { done = true; } else { self.log.push_str(&msg); self.log.push('\n'); }
            }
            if done { self.running = false; self.rx = None; }
        }

        // Dark theme
        ctx.set_visuals(egui::Visuals::dark());

        // Top bar
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("🍿 Pupcorn").color(egui::Color32::from_rgb(255, 180, 60)).size(22.0));
                ui.label(egui::RichText::new("Bug-Bounty Recon Assistant").color(egui::Color32::GRAY).italics());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.running {
                        ui.spinner();
                        ui.label(egui::RichText::new("Running…").color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("Ready").color(egui::Color32::GREEN));
                    }
                });
            });
            ui.add_space(4.0);
        });

        // Left sidebar: tools
        egui::SidePanel::left("tools_panel").min_width(260.0).show(ctx, |ui| {
            ui.heading("Tools");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for t in &self.tools {
                    ui.label(egui::RichText::new(t).size(11.0));
                }
            });
        });

        // Central
        egui::CentralPanel::default().show(ctx, |ui| {
            // Goal input row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Target / Goal:").strong());
                let resp = ui.text_edit_singleline(&mut self.goal);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.goal.is_empty() && !self.running {
                    // trigger run
                }
                let run_btn = ui.add_enabled(!self.running && !self.goal.is_empty(), egui::Button::new("▶ Run Recon"));
                if run_btn.clicked() {
                    self.start_run(ctx);
                }
                if ui.button("🧹 Clear").clicked() {
                    self.log.clear();
                }
            });
            ui.separator();

            // Tabs
            ui.horizontal(|ui| {
                for (i, name) in ["Log", "Info"].iter().enumerate() {
                    if ui.selectable_label(self.selected_tab == i, *name).clicked() {
                        self.selected_tab = i;
                    }
                }
            });
            ui.separator();

            if self.selected_tab == 0 {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.log).font(egui::TextStyle::Monospace).desired_rows(25).desired_width(f32::INFINITY));
                });
            } else {
                ui.label("Pupcorn v0.1 — Rust desktop recon assistant.");
                ui.label("Modules: subdomain, portscan, http_probe, tech_fingerprint, content_discovery, js_analysis, param_discovery, dns_records, whois_rdap, wayback_urls, ip_info, dork_gen.");
                ui.label("LLM: OpenAI-compatible (set OPENAI_BASE_URL/API_KEY/MODEL). Falls back to built-in pipeline.");
            }
        });

        if self.running { ctx.request_repaint(); }
    }
}

impl PupcornApp {
    fn start_run(&mut self, _ctx: &egui::Context) {
        self.running = true;
        self.log.push_str(&format!("=== Starting recon on {} ===\n", self.goal));
        let goal = self.goal.clone();
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let engine = default_engine();
                let _ = tx.send(format!("Starting AI recon on {} ...\n", goal));
                let llm = OpenAiClient::from_env();
                let agent = Agent::new(&llm, &engine);
                let goal_prompt = format!(
                    "Perform full reconnaissance on target '{}'. \
                     Run http_probe, subdomain_enum, port_scan, tech_fingerprint, content_discovery, dns_records, whois_rdap, wayback_urls, ip_info, dork_gen. \
                     Summarize findings concisely.",
                    goal
                );
                match agent.run(&goal_prompt).await {
                    Ok(ans) if !ans.trim().is_empty() => {
                        let _ = tx.send(format!("Agent: {}\n", ans));
                    }
                    _ => {
                        let _ = tx.send("LLM unavailable; running built-in pipeline.\n".to_string());
                        run_pipeline(&engine, &goal, &tx).await;
                    }
                }
                let _ = tx.send("Recon complete.\n".to_string());
                let _ = tx.send("__DONE__".to_string());
            });
        });
    }
}

async fn run_pipeline(engine: &pupcorn_core::ToolEngine, goal: &str, tx: &mpsc::Sender<String>) {
    let tools = ["http_probe", "subdomain_enum", "port_scan", "tech_fingerprint", "content_discovery", "dns_records", "whois_rdap", "wayback_urls", "ip_info", "dork_gen"];
    for t in tools {
        let _ = tx.send(format!("→ {}...\n", t));
        if let Ok(out) = engine.run_tool(t, goal, serde_json::json!({}), None).await {
            let _ = tx.send(format!("  {}: {}\n", t, out.summary));
            if let Some(arr) = out.data.get("subdomains").and_then(|v| v.as_array()) {
                for s in arr { let _ = tx.send(format!("     - {}\n", s.as_str().unwrap_or(""))); }
            }
            if let Some(arr) = out.data.get("open_ports").and_then(|v| v.as_array()) {
                let ports: Vec<String> = arr.iter().map(|p| p.to_string()).collect();
                if !ports.is_empty() { let _ = tx.send(format!("     open: {}\n", ports.join(", "))); }
            }
            if let Some(arr) = out.data.get("technologies").and_then(|v| v.as_array()) {
                for t in arr { let _ = tx.send(format!("     - {}\n", t.as_str().unwrap_or(""))); }
            }
            if let Some(arr) = out.data.get("paths").and_then(|v| v.as_array()) {
                for p in arr {
                    let _ = tx.send(format!("     - {} ({})\n", p.get("path").and_then(|x| x.as_str()).unwrap_or(""), p.get("status").unwrap_or(&serde_json::json!(""))));
                }
            }
            if let Some(arr) = out.data.get("urls").and_then(|v| v.as_array()) {
                for u in arr.iter().take(20) { let _ = tx.send(format!("     - {}\n", u.as_str().unwrap_or(""))); }
                if arr.len() > 20 { let _ = tx.send(format!("     ... {} more\n", arr.len() - 20)); }
            }
            if let Some(arr) = out.data.get("dorks").and_then(|v| v.as_array()) {
                for d in arr { let _ = tx.send(format!("     - {}\n", d.as_str().unwrap_or(""))); }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 720.0]).with_title("Pupcorn — Bug Bounty Recon"),
        ..Default::default()
    };
    eframe::run_native(
        "Pupcorn",
        options,
        Box::new(|cc| Ok(Box::new(PupcornApp::new(cc)))),
    )
}