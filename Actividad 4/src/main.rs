use sysinfo::{System, Networks, Components};
use std::io::Write;
use chrono::Local;
use heim::disk;
use futures::stream::StreamExt;
use tokio;
use heim::units::information::byte;
use serde::Serialize;
use std::fs::{OpenOptions, create_dir_all};
use std::path::Path;

#[tokio::main]
async fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Obtener información de CPU
    let cpu_total = sys.global_cpu_info().cpu_usage();
    let cpu_frecuencia = sys.global_cpu_info().frequency();

    // Obtener uso por núcleo
    let cpu_nucleos: Vec<String> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| format!("Núcleo {}: {:.2}%", i, cpu.cpu_usage()))
        .collect();

    // Obtener información de memoria
    let memoria_usada = sys.used_memory();
    let memoria_total = sys.total_memory();
    let memoria_swap_usada = sys.used_swap();
    let memoria_swap_total = sys.total_swap();
    let memoria_cache = sys.free_memory();

    // Obtener información de red
    let networks = Networks::new_with_refreshed_list();
    let mut total_red_recibido = 0;
    let mut total_red_enviado = 0;

    for (_, data) in &networks {
        total_red_recibido += data.total_received();
        total_red_enviado += data.total_transmitted();
    }

    // Convertir bytes a MB
    let red_recibida_mb = total_red_recibido as f64 / (1024.0 * 1024.0);
    let red_enviada_mb = total_red_enviado as f64 / (1024.0 * 1024.0);

    // Obtener temperatura de componentes
    let components = Components::new_with_refreshed_list();
    let temperaturas: Vec<String> = components
        .iter()
        .map(|c| format!("{}: {:.2}°C", c.label(), c.temperature()))
        .collect();

    // Obtener información de disco de forma asíncrona
    let mut disco_lecturas = 0;
    let mut disco_escrituras = 0;
    
    let mut disk_stream = disk::io_counters().await.unwrap();
    while let Some(Ok(disk)) = disk_stream.next().await {
        disco_lecturas += disk.read_bytes().get::<byte>();  
        disco_escrituras += disk.write_bytes().get::<byte>();
    }

    // Convertir bytes a MB
    let disco_lecturas_mb = disco_lecturas as f64 / (1024.0 * 1024.0);
    let disco_escrituras_mb = disco_escrituras as f64 / (1024.0 * 1024.0);

    // Obtener los top 5 procesos que más CPU consumen
    let mut procesos: Vec<_> = sys.processes().values().collect();
    procesos.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

    let top_5_procesos: Vec<String> = procesos.iter().take(5).map(|proc| {
        format!(
            "{} (PID {}): {:.2}% CPU, {} KB memoria",
            proc.name(),
            proc.pid(),
            proc.cpu_usage(),
            proc.memory()
        )
    }).collect();

    #[derive(Serialize)]
    struct Metrics {
        timestamp: String,
        cpu_total: f32,
        cpu_freq_mhz: u64,
        cpu_nucleos: Vec<String>,
        memoria_usada_mb: u64,
        memoria_total_mb: u64,
        memoria_swap_usada_mb: u64,
        memoria_swap_total_mb: u64,
        memoria_cache_mb: u64,
        red_recibida_mb: f64,
        red_enviada_mb: f64,
        disco_lecturas_mb: f64,
        disco_escrituras_mb: f64,
        temperaturas: Vec<String>,
        top_procesos: Vec<String>,
    }

    let metrics = Metrics {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        cpu_total,
        cpu_freq_mhz: cpu_frecuencia,
        cpu_nucleos,
        memoria_usada_mb: memoria_usada / 1024,
        memoria_total_mb: memoria_total / 1024,
        memoria_swap_usada_mb: memoria_swap_usada / 1024,
        memoria_swap_total_mb: memoria_swap_total / 1024,
        memoria_cache_mb: memoria_cache / 1024,
        red_recibida_mb,
        red_enviada_mb,
        disco_lecturas_mb,
        disco_escrituras_mb,
        temperaturas,
        top_procesos: top_5_procesos,
    };

    let json_line = serde_json::to_string(&metrics).unwrap();

    //let mut file = OpenOptions::new()
    //.append(true)
    //.create(true)
    //.open("C:\\Monitoreo\\metrics.jsonl")
    //.unwrap();
    let ruta_carpeta = "C:\\Monitoreo";
    let ruta_archivo = format!("{}\\metrics.jsonl", ruta_carpeta);

    // Verificar y crear carpeta si no existe
    if !Path::new(ruta_carpeta).exists() {
        create_dir_all(ruta_carpeta).expect("No se pudo crear la carpeta Monitoreo");
    }

    // Abrir o crear el archivo en modo append
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&ruta_archivo)
        .expect("No se pudo abrir o crear el archivo metrics.jsonl");

    writeln!(file, "{}", json_line).unwrap();
}