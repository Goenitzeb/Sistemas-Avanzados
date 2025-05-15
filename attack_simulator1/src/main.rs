use eframe::{egui, App, Frame};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::net::UdpSocket;
use serde_json::json;
use rand::Rng;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use serde::Serialize;

#[derive(PartialEq, Clone, Debug)]
enum AttackType {
    None,
    DDoS,
    CPUSpike,
    MemoryLeak,
    Custom,
}

#[derive(Serialize)]
struct AttackReport {
    tipo: String,
    inicio: String,
    fin: String,
}

fn guardar_reporte(reporte: &AttackReport) {
    // JSON
    let mut archivo_json = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log_ataques.json")
        .unwrap();
    let linea = serde_json::to_string(&reporte).unwrap();
    writeln!(archivo_json, "{}", linea).unwrap();

    // CSV
    let mut archivo_csv = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log_ataques.csv")
        .unwrap();
    writeln!(archivo_csv, "{},{},{}", reporte.tipo, reporte.inicio, reporte.fin).unwrap();
}

struct AttackApp {
    selected_attack: AttackType,
    intensity: u32,
    log: Arc<Mutex<Vec<String>>>,
    attack_running: bool,
}

impl Default for AttackApp {
    fn default() -> Self {
        Self {
            selected_attack: AttackType::None,
            intensity: 50,
            log: Arc::new(Mutex::new(Vec::new())),
            attack_running: false,
        }
    }
}

impl App for AttackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simulador de Ataques");

            ui.horizontal(|ui| {
                ui.label("Tipo de ataque:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.selected_attack))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_attack, AttackType::DDoS, "DDoS");
                        ui.selectable_value(&mut self.selected_attack, AttackType::CPUSpike, "CPU Spike");
                        ui.selectable_value(&mut self.selected_attack, AttackType::MemoryLeak, "Fuga de Memoria");
                        ui.selectable_value(&mut self.selected_attack, AttackType::Custom, "Combinado");
                    });
            });

            ui.add(egui::Slider::new(&mut self.intensity, 1..=100).text("Intensidad"));

            if ui.button(if self.attack_running { "Detener Ataque" } else { "Iniciar Ataque" }).clicked() {
                if self.attack_running {
                    self.attack_running = false;
                    self.log.lock().unwrap().push("Ataque detenido.".to_string());
                } else {
                    self.attack_running = true;
                    let log = self.log.clone();
                    let tipo = self.selected_attack.clone();
                    let intensidad = self.intensity;

                    thread::spawn(move || {
                        let inicio = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        log.lock().unwrap().push(format!("Ataque {:?} iniciado con intensidad {}.", tipo, intensidad));

                        match tipo {
                            AttackType::DDoS => run_ddos(log.clone(), intensidad),
                            AttackType::CPUSpike => run_cpu_spike(log.clone(), intensidad),
                            AttackType::MemoryLeak => run_memory_leak(log.clone(), intensidad),
                            AttackType::Custom => {
                                run_ddos(log.clone(), intensidad / 2);
                                run_cpu_spike(log.clone(), intensidad / 2);
                            }
                            AttackType::None => (),
                        }

                        thread::sleep(Duration::from_secs(10));
                        let fin = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        let reporte = AttackReport {
                            tipo: format!("{:?}", tipo),
                            inicio,
                            fin,
                        };
                        guardar_reporte(&reporte);
                    });
                }
            }

            ui.separator();
            ui.label("Log de Actividad:");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for entry in self.log.lock().unwrap().iter() {
                    ui.label(entry);
                }
            });
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn run_ddos(log: Arc<Mutex<Vec<String>>>, intensity: u32) {
    let server_addr = "127.0.0.1:4000";

    for i in 0..intensity {
        let log = log.clone();
        let server = server_addr.to_string();

        thread::spawn(move || {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("No se pudo abrir socket UDP");
            let mut rng = rand::thread_rng();

            for _ in 0..100 {
                let data = json!({
                    "cpu": rng.gen_range(10..30),
                    "network": format!("{} MB/s", rng.gen_range(50..100)),
                    "memory": rng.gen_range(30..50),
                });

                let payload = data.to_string();
                let _ = socket.send_to(payload.as_bytes(), &server);
                thread::sleep(Duration::from_millis(10));
            }

            log.lock().unwrap().push(format!("Hilo DDoS #{} completó envío de 100 paquetes.", i));
        });
    }

    log.lock().unwrap().push(format!("Ataque DDoS lanzado con {} hilos.", intensity));
}

fn run_cpu_spike(log: Arc<Mutex<Vec<String>>>, intensity: u32) {
    send_metrics_udp(95, 10, 30);
    for i in 0..intensity {
        let _ = thread::spawn(move || {
            let mut x = 0u64;
            loop {
                x = x.wrapping_add(1);
                if x % 1_000_000_000 == 0 {
                    println!("CPU thread #{} spinning", i);
                }
            }
        });
    }
    log.lock().unwrap().push(format!("{} hilos de CPU lanzados.", intensity));
}

fn run_memory_leak(log: Arc<Mutex<Vec<String>>>, intensity: u32) {
    thread::spawn(move || {
        let mut data = Vec::new();
        for i in 0..intensity {
            data.push(vec![0u8; 10_000_000]);
            thread::sleep(Duration::from_millis(500));
            log.lock().unwrap().push(format!("Fuga de memoria paso {}", i));
        }
        std::mem::forget(data);
    });
}

fn send_metrics_udp(cpu: u32, network_mb: u32, memory_percent: u32) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("No se pudo abrir socket UDP");
    let server_addr = "127.0.0.1:4000";

    let data = json!({
        "cpu": cpu,
        "network": format!("{} MB/s", network_mb),
        "memory": memory_percent
    });

    let payload = data.to_string();
    let _ = socket.send_to(payload.as_bytes(), server_addr);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Simulador de Ataques",
        options,
        Box::new(|_cc| Box::new(AttackApp::default())),
    )
}
