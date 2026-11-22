import shutil
import subprocess

import psutil


def sample_metrics() -> dict:
    memory = psutil.virtual_memory()
    result = {"type": "telemetry", "cpu": psutil.cpu_percent(interval=0.1), "ram": memory.percent,
              "ram_used": memory.used / 1048576, "ram_total": memory.total / 1048576}
    executable = shutil.which("nvidia-smi")
    if executable:
        try:
            completed = subprocess.run(
                [executable, "--query-gpu=utilization.gpu,memory.used,memory.total",
                 "--format=csv,noheader,nounits"], capture_output=True, text=True,
                timeout=1.5, check=True, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            gpu, used, total = map(float, completed.stdout.splitlines()[0].split(","))
            if 0 <= gpu <= 100 and 0 <= used <= total and total > 0:
                result.update(gpu=gpu, vram_used=used, vram_total=total)
        except (OSError, subprocess.SubprocessError, ValueError, IndexError):
            pass  # Missing metrics remain unavailable; never invent zero usage.
    return result
