# 🎴 Chkobba CLI (Rust)

A fully native, multi-threaded, cross-platform terminal implementation of the classic Tunisian card game **Chkobba (شكبة)**. Built entirely from scratch in Rust, featuring real-time keyboard navigation and an immersive embedded audio system.

---

## 🚀 Features

* 🎮 **Full Keyboard Navigation:** No clunky text inputs. Use your **Arrow Keys** to browse your hand and **Enter** to drop a card.
* 🔊 **Embedded Audio Pipeline:** Sound effects (table slams, crowd cheers, and defeat violins) are baked directly into the executable file. Zero external dependencies.
* 🎯 **Interactive Multi-Choice Selector:** Total tactical control. When playing a card that yields multiple targets or summation combinations, the game dynamically prompts you to select your exact target (e.g., choosing single targets vs. multi-card combos like snatching the *Sabʿa l-ḥayya* $7♦$).
* 🧠 **Dynamic Difficulty Adjustment (DDA):** Seamlessly adjusts to your skill level across sessions.
  - **Easy Mode:** Features a subtle probability margin for the CPU to occasionally make human-like mistakes or suboptimal plays.
  - **Hard Mode:** Automatically unlocks once you hit a minimum of **5 matches played** and maintain a **$\ge 65\%$ win rate**.
  - **Adaptive CPU AI:** The CPU opponent evaluates all board combinations and strategically picks its best capture route based on the active difficulty mode.
* 🧹 **Polished Terminal UI:** Context-aware prompts, dynamic card styling, and clean thematic prints for round summaries, candidate captures, and Final Sweeps.
* 🔇 **Hardware Mute Switch:** Dynamically toggle all audio on/off by pressing `M` mid-game to stream your own music playlist without layout glitches.
* 🤖 **Smart CPU Opponent:** Face off against a script that dynamically calculates matches, sweeps, and table clears.
* 📊 **Persistent Statistics:** Tracks your win/loss streaks across sessions.

---

## 🕹️ Controls

| Key | Action |
| :--- | :--- |
| `Left / Up Arrow` | Move selection left |
| `Right / Down Arrow`| Move selection right |
| `Enter` | Confirm card selection / drop card |
| `M` / `m` | Toggle audio mute (`[MUTED 🔇]` display) |
| `Esc` | Safely exit the game |

---

## 📦 How to Download & Play

You do **not** need to install Rust or any build tools to play this game. Ready-to-run binaries are automatically compiled for your system.

1. Go to the **Actions** tab (or Releases page) of this repository.
2. Download the artifact zip file matching your operating system:
   * **Windows:** `chkobba-windows.exe`
   * **Linux:** `chkobba-linux`
3. Extract the file, open your terminal (like Alacritty, PowerShell, or bash), and run the executable!

---

## 🔧 Local Development & Building

If you want to modify the code or build it from source locally:

### Prerequisites (Linux Only)
Because the game uses a native multi-threaded audio framework (`rodio`), Linux developers need ALSA development headers installed:
```bash
sudo apt install libasound2-dev

## Build & Run
git clone [https://github.com/Aymen27k/chkobba-rust.git](https://github.com/Aymen27k/chkobba-rust.git)
cd chkobba-rust
cargo run --release
```
---

## 📜 Chkobba Scoring Quick Ref

This engine was meticulously built following the rules outlined in the official game documentation. For an in-depth breakdown of gameplay, check out the [Official Chkobba Game Rules](https://officialgamerules.org/game-rules/chkobba/).

---

## 👥 Credits & Attribution

* 🛠️ **Developer:** Developed with dedication and persistence by **Aymen Kalaï Ezar** — bringing classic Tunisian culture into the terminal.
* 🔊 **Audio Assets:** Sound effects sourced from [Pixabay](https://pixabay.com/) under the Pixabay Content License.