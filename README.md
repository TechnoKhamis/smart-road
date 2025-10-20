# 🚗 Smart Road Simulation

## Overview
**Smart Road** is a simulation of an autonomous vehicle (AV) intersection management system — a "smart" traffic control strategy without traffic lights.  
The goal is to minimize congestion and collisions using physics-based logic and intelligent coordination.

---

## 🎯 Objectives
- Simulate a cross-intersection with right, straight, and left-turn lanes.
- Implement a smart algorithm that controls AV movement safely and efficiently.
- Include animations and interactive input to spawn vehicles dynamically.
- Display simulation statistics upon completion.

---

## 🧱 Project Structure

```
smart_road/
├── src/
│   ├── main.rs
│   ├── simulation/
│   │   ├── mod.rs
│   │   ├── vehicle.rs
│   │   ├── intersection.rs
│   │   └── physics.rs
│   ├── render/
│   │   ├── mod.rs
│   │   ├── assets.rs
│   │   └── animation.rs
│   ├── events/
│   │   ├── mod.rs
│   │   └── input.rs
│   └── stats/
│       └── mod.rs
└── assets/
    ├── cars/
    └── roads/
```

---

## 📁 File Descriptions

### 🏁 `src/main.rs`
The entry point of the simulation. Initializes SDL2, manages the game loop, handles event routing, and displays statistics when the simulation ends.

---

### ⚙️ `src/simulation/`
Handles all logic related to how vehicles move, interact, and navigate through the intersection.

- **`mod.rs`** — Central module linking all simulation components.  
- **`vehicle.rs`** — Defines the `Vehicle` struct (position, route, velocity, etc.) and handles movement/safety logic.  
- **`intersection.rs`** — Manages lane layout, entry/exit logic, and the smart traffic algorithm.  
- **`physics.rs`** — Implements physical calculations (velocity, distance, time, and safe distances).  

---

### 🎨 `src/render/`
Responsible for drawing and animating vehicles and the environment.

- **`mod.rs`** — Exports render functions and manages canvas setup.  
- **`assets.rs`** — Loads and manages textures for cars and roads.  
- **`animation.rs`** — Handles vehicle animation (rotation, smooth movement, and turning).  

---

### ⌨️ `src/events/`
Manages keyboard and user input for controlling the simulation.

- **`mod.rs`** — Connects event handling logic.  
- **`input.rs`** — Handles keyboard controls:  
  - Arrow keys to spawn vehicles.  
  - `R` for continuous random vehicle generation.  
  - `Esc` to end simulation and display statistics.  

---

### 📊 `src/stats/`
Handles data collection and reporting.

- **`mod.rs`** — Tracks and displays:  
  - Max/min velocity and time.  
  - Number of vehicles passed.  
  - Close-call (safety distance) violations.  

---

### 🖼 `assets/`
Holds visual assets for rendering the simulation.

- **`cars/`** — Car sprite images (different models, turning animations).  
- **`roads/`** — Road and intersection textures.

#### 🔗 Recommended Asset Sources
- [Limezu](https://limezu.itch.io/)  
- [FinalBossBlue](https://finalbossblues.itch.io/)  
- [MobileGameGraphics](https://mobilegamegraphics.com/)  
- [The Spriters Resource](https://www.spriters-resource.com/)

---

## 🧮 Statistics Collected
- Max/Min vehicle velocity.  
- Max/Min time to pass intersection.  
- Total vehicles passed.  
- Number of close calls (unsafe distances).  

---

## 🎮 Controls
| Key | Action |
|-----|---------|
| ↑ | Spawn vehicle from South to North |
| ↓ | Spawn vehicle from North to South |
| ← | Spawn vehicle from East to West |
| → | Spawn vehicle from West to East |
| R | Auto-generate random vehicles |
| Esc | End simulation and show stats |

---

## 🧠 Concepts Learned
- Rust and SDL2 integration  
- Basic physics modeling  
- Event-driven simulation loops  
- Animation and coordinate transformations  
- Data collection and visualization  

---

## 🏁 Bonus Ideas
- Add acceleration/deceleration to simulate more realistic physics.  
- Create custom car sprites.  
- Expand statistics visualization with graphs or charts.  

---

© 2025 Smart Road Simulation by Rashid
