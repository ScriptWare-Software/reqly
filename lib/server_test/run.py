import subprocess
import sys

def run_script(script_path):
    try:
        subprocess.Popen([sys.executable, script_path])
    except Exception as e:
        print(f"Error running script {script_path}: {e}")
        
if __name__ == "__main__":
    scripts = [
        "socketio_script.py",
        "tcp.py",
        "udp.py",
        "wss.py",
    ]

    processes = []

    for script in scripts:
        process = run_script(script)
        processes.append(process)

    print("All scripts have been launched. Press Ctrl+C to exit.")

    try:
        while True:
            pass
    except KeyboardInterrupt:
        for process in processes:
            process.terminate()
        print("All scripts have been terminated.")