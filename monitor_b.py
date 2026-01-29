import requests
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import time

# Настройки подключения
API_URL = "http://127.0.0.1:3031/state"

# Подготовка окна графиков
plt.style.use('dark_background')
fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(10, 10))
fig.canvas.manager.set_window_title('Himera Him: Global Ledger & AI Monitor')

history = {
    'fund': [],
    'my_balance': [],
    'weights': [[] for _ in range(10)],
    'time': []
}

def animate(i):
    try:
        response = requests.get(API_URL, timeout=1)
        data = response.json()
        
        fund = data['neural_fund']
        weights = data['ai_weights']
        # Берем первый доступный баланс из словаря (твой кошелек)
        balances = data['balances']
        my_addr = list(balances.keys())[0]
        my_balance = balances[my_addr]
        
        # Обновляем историю
        history['time'].append(time.time())
        history['fund'].append(fund)
        history['my_balance'].append(my_balance)
        for idx, w in enumerate(weights):
            history['weights'][idx].append(w)

        # Лимит истории (50 точек)
        if len(history['time']) > 50:
            history['time'].pop(0)
            history['fund'].pop(0)
            history['my_balance'].pop(0)
            for h in history['weights']: h.pop(0)

        # 1. График Фонда
        ax1.clear()
        ax1.plot(history['fund'], color='#00ffcc', label='Neural Fund (HIM)')
        ax1.set_title(f"Neural Fund: {fund:.4f} HIM", color='#00ffcc')
        ax1.fill_between(range(len(history['fund'])), history['fund'], color='#00ffcc', alpha=0.1)
        ax1.legend(loc='upper left')

        # 2. График Личного Баланса
        ax2.clear()
        ax2.plot(history['my_balance'], color='#ffcc00', label='Your Wallet Balance')
        ax2.set_title(f"My Balance: {my_balance:.2f} HIM | Addr: {my_addr[:12]}...", color='#ffcc00')
        ax2.set_ylabel("HIM Tokens")
        ax2.legend(loc='upper right')

        # 3. График Весов ИИ
        ax3.clear()
        for idx, h in enumerate(history['weights']):
            ax3.plot(h, alpha=0.6)
        ax3.set_title("AI Neural Weights (Proof-of-Learning)", color='#ff00ff')

    except Exception as e:
        print(f"Синхронизация с Himera Core... {e}")

ani = FuncAnimation(fig, animate, interval=1000)
plt.tight_layout()
plt.show()
