import matplotlib.pyplot as plt
import pandas as pd
import json
from pathlib import Path

import numpy as np
import seaborn as sns
from sklearn.linear_model import LinearRegression

from jinja2 import Template

# Leer archivo JSONL
data = []
with open("metrics.jsonl", "r", encoding="utf-8") as f:
    for line in f:
        data.append(json.loads(line))

# Convertir a DataFrame
df = pd.DataFrame(data)

# Convertir timestamps a objetos datetime
df["timestamp"] = pd.to_datetime(df["timestamp"], format="%Y-%m-%d %H:%M:%S")

# Ordenar por tiempo
df = df.sort_values("timestamp")

# Gráfica de uso de CPU
plt.figure(figsize=(10, 5))
plt.plot(df["timestamp"], df["cpu_total"], marker='o', linestyle='-')
plt.title("Uso de CPU (%)")
plt.xlabel("Tiempo")
plt.ylabel("CPU (%)")
plt.xticks(rotation=45)
plt.grid(True)
plt.tight_layout()
plt.savefig("cpu_usage.png")
plt.show()

# Gráfica de uso de Memoria
plt.figure(figsize=(10, 5))
plt.plot(df["timestamp"], df["memoria_usada_mb"], marker='o', linestyle='-', label="Usada")
plt.plot(df["timestamp"], df["memoria_total_mb"], linestyle='--', label="Total")
plt.title("Uso de Memoria (MB)")
plt.xlabel("Tiempo")
plt.ylabel("Memoria (MB)")
plt.xticks(rotation=45)
plt.legend()
plt.grid(True)
plt.tight_layout()
plt.savefig("memoria_usage.png")
plt.show()


# Red
plt.figure(figsize=(12, 6))
plt.plot(df["timestamp"], df["red_recibida_mb"], label="Red recibida (MB)")
plt.plot(df["timestamp"], df["red_recibida_mb"], label="Red enviada (MB)")
plt.ylabel("Red (MB)")
plt.xlabel("Tiempo")
plt.title("Uso de Red")
plt.grid(True)
plt.legend()
plt.tight_layout()
plt.savefig("red_usage.png")
plt.show()

# Disco
plt.figure(figsize=(12, 6))
plt.plot(df["timestamp"], df["disco_lecturas_mb"], label="Lecturas (MB)")
plt.plot(df["timestamp"], df["disco_escrituras_mb"], label="Escrituras (MB)")
plt.ylabel("Disco (MB)")
plt.xlabel("Tiempo")
plt.title("Actividad de Disco")
plt.grid(True)
plt.legend()
plt.tight_layout()
plt.savefig("disk_usage.png")
plt.show()

# Mostrar tabla de procesos más exigentes por cada punto
print("Top 5 procesos por uso de CPU en cada intervalo:\n")
for i, row in df.iterrows():
    print(f"\n{row['timestamp']}:\n")
    for proc in row["top_procesos"]:
        print(f"  - {proc}")

# Crear directorio para guardar tablas HTML
output_dir = Path("html_output")
output_dir.mkdir(exist_ok=True)

html_sections = []

# Generar una tabla HTML por cada intervalo
for _, row in df.iterrows():
    timestamp_str = row["timestamp"].strftime("%Y-%m-%d %H:%M:%S")
    procesos = row["top_procesos"]
    procesos_df = pd.DataFrame({"Proceso": procesos})

    html = f"<h3>{timestamp_str}</h3>"
    html += procesos_df.to_html(index=False, escape=False, border=1)
    html_sections.append(html)

# Juntar todo en un solo archivo HTML
final_html = f"""
<html>
<head>
    <meta charset="UTF-8">
    <title>Top 5 procesos por uso de CPU</title>
    <style>
        body {{ font-family: Arial, sans-serif; padding: 20px; }}
        h3 {{ color: #444; }}
        table {{
            border-collapse: collapse;
            margin-bottom: 30px;
            width: 100%;
        }}
        th, td {{
            border: 1px solid #ccc;
            padding: 8px;
            text-align: left;
        }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>Top 5 procesos por uso de CPU</h1>
    {''.join(html_sections)}
</body>
</html>
"""

# Guardar archivo
output_path = output_dir / "top_procesos.html"
with open(output_path, "w", encoding="utf-8") as f:
    f.write(final_html)

print(f"Archivo HTML generado: {output_path.absolute()}")


# ----------- Estadísticas básicas -------------
print("\nEstadísticas de uso:")

for col in ["cpu_total", "memoria_usada_mb", "red_recibida_mb", "red_enviada_mb", "disco_lecturas_mb", "disco_escrituras_mb"]:
    print(f" {col}: Promedio = {df[col].mean():.2f}, Pico = {df[col].max():.2f}")

# ----------- Correlaciones básicas -------------
print("\nCorrelaciones entre variables:")
corr = df[["cpu_total", "memoria_usada_mb", "red_recibida_mb", "disco_lecturas_mb"]].corr()
print(corr)

# Detección básica de proceso "chrome"
df["usa_chrome"] = df["top_procesos"].apply(lambda procesos: any("chrome" in p.lower() for p in procesos))
chrome_vs_cpu = df.groupby("usa_chrome")["cpu_total"].mean()
print("\nUso de CPU según Chrome activo:")
print(chrome_vs_cpu)

# ----------- Proyección de llenado de swap -------------
pendiente = 0
if df["memoria_swap_usada_mb"].max() > 0:
    df["timestamp_ordinal"] = df["timestamp"].apply(lambda x: x.toordinal())
    X = df["timestamp_ordinal"].values.reshape(-1, 1)
    y = df["memoria_swap_usada_mb"].values

    model = LinearRegression().fit(X, y)
    pendiente = model.coef_[0]

    if pendiente > 0:
        dias_para_lleno = (df["memoria_swap_total_mb"].iloc[0] - y[-1]) / pendiente
        fecha_lleno = df["timestamp"].iloc[-1] + pd.Timedelta(days=dias_para_lleno)
        print(f"\nProyección: El swap se llenará en {dias_para_lleno:.2f} días, aprox. el {fecha_lleno.date()}")
    else:
        print("\nEl uso de swap no está creciendo.")
else:
    print("\nNo se está usando swap en este periodo.")

# ----------- Horarios críticos -------------
df["hora"] = df["timestamp"].dt.hour
df["dia_semana"] = df["timestamp"].dt.day_name()

print("\nUso promedio de red por día/hora:")
pivot = df.pivot_table(index="dia_semana", columns="hora", values="red_recibida_mb", aggfunc="mean")
print(pivot.fillna(0).round(2))

# Heatmap
plt.figure(figsize=(12, 6))
sns.heatmap(pivot.fillna(0), cmap="YlOrRd", linewidths=0.5, annot=True, fmt=".1f")
plt.title("Uso de red por día/hora (MB)")
plt.tight_layout()
plt.savefig("uso_red_dia_hora.png")
plt.show()

# Secciones HTML para incrustar
estadisticas_html = "<ul>"
for col in ["cpu_total", "memoria_usada_mb", "red_recibida_mb", "red_enviada_mb", "disco_lecturas_mb", "disco_escrituras_mb"]:
    estadisticas_html += f"<li><strong>{col}</strong>: Promedio = {df[col].mean():.2f}, Pico = {df[col].max():.2f}</li>"
estadisticas_html += "</ul>"

# Correlación
correlaciones_html = df[["cpu_total", "memoria_usada_mb", "red_recibida_mb", "disco_lecturas_mb"]].corr().round(2).to_html()

# Uso de Chrome
chrome_html = chrome_vs_cpu.to_frame(name="Promedio CPU").reset_index().to_html(index=False)

# Proyección swap
if pendiente > 0:
    swap_html = f"<p><strong>Proyección:</strong> El swap se llenará en {dias_para_lleno:.2f} días, aproximadamente el <strong>{fecha_lleno.date()}</strong>.</p>"
else:
    swap_html = "<p> El uso de swap no está creciendo.</p>"

# Heatmap ya fue guardado como uso_red_dia_hora.png

# Incluir el HTML de los procesos
with open(output_path, "r", encoding="utf-8") as f:
    procesos_html = f.read()

# Usar plantilla con Jinja2
template = Template("""
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <title>Reporte de Monitoreo del Sistema</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; background-color: #f8f8f8; }
        h1, h2 { color: #333; }
        .section { margin-bottom: 40px; background: #fff; padding: 20px; border-radius: 10px; box-shadow: 0 0 10px #ddd; }
        img { border: 1px solid #ccc; padding: 5px; background: white; }
        table { border-collapse: collapse; width: 100%; margin-top: 10px; }
        th, td { border: 1px solid #ccc; padding: 8px; text-align: left; }
        th { background-color: #f0f0f0; }
    </style>
</head>
<body>
    <h1>Reporte de Monitoreo del Sistema</h1>

    <div class="section">
        <h2>1. Estadísticas Generales</h2>
        {{ estadisticas_html | safe }}
    </div>

    <div class="section">
        <h2>2. Correlaciones</h2>
        {{ correlaciones_html | safe }}
    </div>

    <div class="section">
        <h2>3. Impacto del uso de Chrome</h2>
        {{ chrome_html | safe }}
    </div>

    <div class="section">
        <h2>4. Proyección del uso de Swap</h2>
        {{ swap_html | safe }}
    </div>

    <div class="section">
        <h2>5. Uso por Intervalos</h2>
        {{ procesos_html | safe }}
    </div>

</body>
</html>
""")

# Renderizar plantilla
rendered = template.render(
    estadisticas_html=estadisticas_html,
    correlaciones_html=correlaciones_html,
    chrome_html=chrome_html,
    swap_html=swap_html,
    procesos_html=procesos_html,
)

# Guardar el reporte
final_report_path = output_dir / "reporte_completo.html"
with open(final_report_path, "w", encoding="utf-8") as f:
    f.write(rendered)

print(f"Reporte HTML completo generado: {final_report_path.absolute()}")