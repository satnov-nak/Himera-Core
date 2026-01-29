import requests
import time
import os

NODES = [
    "http://127.0.0.1:3030/state",
    "http://127.0.0.1:3031/state",
    "http://127.0.0.1:3032/state"
]

def draw_gauge(value, label, max_val=1.0, color="\033[92m"):
    size = 20
    filled = int((min(value, max_val) / max_val) * size)
    bar = "█" * filled + "░" * (size - filled)
    reset = "\033[0m"
    return f"{label:15} {color}[{bar}]{reset} {value:.4f}"

def draw_neural_activity(weights):
    # Визуализация 10 весов глобального мозга
    viz = "BRAIN ACTIVITY: "
    for w in weights:
        # Рисуем разные символы в зависимости от "силы" нейрона
        if w < 0.3: char = "."
        elif w < 0.6: char = "o"
        elif w < 0.8: char = "O"
        else: char = "@"
        viz += f"{char} "
    return viz

def main():
    while True:
        os.system('clear' if os.name == 'posix' else 'cls')
        print("\033[94m" + "="*60 + "\033[0m")
        print("   HIMERA HIM v8.2: NEURAL-QUANTUM COMMAND CENTER")
        print("\033[94m" + "="*60 + "\033[0m")
        
        for i, url in enumerate(NODES):
            try:
                r = requests.get(url, timeout=0.5)
                data = r.json()
                
                intel = data.get("intelligence_level", 0.0)
                fund = data.get("neural_fund", 0.0)
                diff = data.get("difficulty", 1.0)
                weights = data.get("global_brain", [])
                
                print(f"\n\033[1mNODE #{i+1} [{url}]\033[0m")
                print(draw_gauge(intel, "INTELLECT", max_val=0.2, color="\033[96m"))
                print(draw_gauge(fund, "NEURAL FUND", max_val=5.0, color="\033[93m"))
                print(f"DIFFICULTY:      {diff:.2f}x")
                print(f"\033[95m{draw_neural_activity(weights)}\033[0m")
                
            except Exception:
                print(f"\nNODE #{i+1}: \033[91m[OFFLINE]\033[0m")

        print("\n" + "\033[94m" + "="*60 + "\033[0m")
        print("Pulse sync: 3000ms | Proof-of-Learning: ACTIVE")
        time.sleep(1)

if __name__ == "__main__":
    main()
