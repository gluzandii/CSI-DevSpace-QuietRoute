# 🚶 Quiet Route - Safe Pedestrian Pathfinding

A safety-aware routing system that finds the safest walking routes by combining real-world street networks with streetlight and police station data.

## 🎯 Problem

Pedestrians, especially vulnerable groups, need safer routes—not just shorter ones. Safety depends on:
- Street lighting coverage
- Proximity to police stations and emergency services
- Time of day (future enhancement)

## 💡 Solution

Quiet Route analyzes nearly **1 million street segments** in Bangalore, assigning each a safety score (0.0-1.0) based on:
- Distance to **196,642 streetlights**
- Distance to **147 police stations**

Then uses intelligent A* pathfinding to find routes that balance **distance vs. safety**.

## 📊 Key Statistics

| Metric | Value |
|--------|-------|
| Street intersections (nodes) | 853,073 |
| Street segments (edges) | 951,787 |
| Streetlights loaded | 196,642 |
| Police stations | 147 |
| Loading time | ~30-60 seconds |
| Routing time | Milliseconds |

## 🏗️ Architecture

```
quiet-route/
├── backend/          → Demo application
├── quiet-core/       → Core routing engine
│   ├── models.rs     → Data structures (Node, Edge, Graph)
│   ├── parser.rs     → OSM data loading
│   ├── safety.rs     → Safety scoring system
│   └── router.rs     → A* pathfinding
└── utils/            → Helper functions
    ├── geo.rs        → Haversine distance
    └── kml.rs        → KML file parsing
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (`rustup install stable`)
- Bengaluru OSM data: `data/OSM (Open Map Data)/bengaluru.osm.pbf`
- Safety KML files in `data/KML (Lights)/` and `data/KML (Police)/`

### Run the Demo

```bash
cargo run --release
```

**Expected Output:**
```
🚀 Quiet Route - Safe Pathfinding System
===============================================

✅ Graph loaded successfully!
   • 853073 nodes (intersections)
   • 951787 edges (street segments)

🧭 Finding Safe Route
📍 Start: Cubbon Park Area
📍 End:   MG Road Area

✅ ROUTE FOUND!
   • Safety-weighted cost: 1508.42
   • Actual distance: 1453.23 meters (1.45 km)
   • Number of waypoints: 84
```

### Run Tests

```bash
# All tests
cargo test

# Specific modules
cargo test parser
cargo test router
cargo test safety

# With output
cargo test -- --nocapture
```

### Generate Documentation

```bash
cargo doc --open
```

This opens comprehensive API documentation in your browser.

## 🧮 How It Works

### 1. Safety Scoring Algorithm

Each street segment gets a safety score based on graduated distance bonuses:

**Streetlight Bonuses:**
- < 150m: +0.35 (excellent)
- 150-300m: +0.25 (good)
- 300-500m: +0.15 (moderate)
- 500-800m: +0.05 (weak)

**Police Station Bonuses:**
- < 500m: +0.25 (very safe)
- 500-1km: +0.15 (safe)
- 1-2km: +0.08 (moderate)
- 2-3km: +0.03 (slight)

**Base score:** 0.5 → **Max score:** 1.0

### 2. Pathfinding Cost Function

```rust
cost = distance × (1.0 / safety_score)
```

**Examples:**
- Safe street (score 1.0): cost = 100m × 1.0 = **100**
- Dangerous street (score 0.1): cost = 100m × 10.0 = **1000**

The A* algorithm naturally avoids dangerous streets when safer alternatives exist!

### 3. A* vs Dijkstra

We use **A*** with a straight-line heuristic instead of Dijkstra because:
- ✅ 20-100x faster for geographic routing
- ✅ Still guarantees optimal path
- ✅ Explores fewer nodes by guessing direction to goal

## 📖 API Integration Example

```rust
use quiet_core::parser::parse_osm;
use quiet_core::router::find_safe_path;

// 1. Load network once at startup
let network = parse_osm("data/bengaluru.osm.pbf")?;

// 2. User requests route
let start = network.find_closest_node(12.976, 77.593)?;
let end = network.find_closest_node(12.975, 77.605)?;

// 3. Find safe path
let (cost, path) = find_safe_path(&network.graph, start, end)?;

// 4. Convert to GeoJSON coordinates
let coords = network.path_to_coords(&path);

// 5. Return as JSON
// { "type": "LineString", "coordinates": [[lon, lat], ...] }
```

## 🔧 Technical Highlights

### Performance Optimizations
1. **Parallel KML parsing** - Uses Rayon to load files on all CPU cores
2. **KD-trees** - O(log n) spatial queries instead of O(n) scans
3. **Binary PBF format** - 10x faster than XML
4. **A* heuristic** - Explores far fewer nodes than Dijkstra

### Data Structures
- **petgraph** - Industry-standard graph library
- **HashMap lookups** - O(1) coordinate retrieval
- **Undirected graph** - Streets are bidirectional

### Real-World Data
- Uses actual municipal data (not synthetic)
- Accurate Haversine distance (accounts for Earth's curvature)
- Graduated safety scoring (not binary safe/unsafe)

## 🎤 Presentation Talking Points

**Problem:** *"Pedestrians need safer routes, not just shorter ones. Safety depends on lighting and proximity to authorities."*

**Solution:** *"We built a routing engine using 196,000+ streetlights and police data to score every street in Bangalore, then find paths optimizing for both distance and safety."*

**Innovation:**
1. Multi-source data fusion (OSM + municipal data)
2. Graduated safety scoring (distance decay model)
3. Performance (KD-trees + parallel processing + A*)
4. Rust (memory-safe, fast, production-ready)

**Demo:** *"This 1.45km route from Cubbon Park to MG Road has a cost/distance ratio of 1.04x, meaning it's very safe. If forced through dangerous areas, that ratio would be 2-3x higher."*

## 📈 Interpreting Results

**Cost vs Distance Ratio:**
- **~1.0x** → Very safe route
- **1.5-2.0x** → Moderate safety concerns
- **>2.0x** → Significant safety compromises needed

**Example:**
```
Safety-weighted cost: 1508.42
Actual distance: 1453.23m
Ratio: 1.04x → Very safe route!
```

## 🚀 Future Enhancements

- [ ] REST API for web/mobile apps
- [ ] Time-aware routing (lights only matter at night)
- [ ] Accident data integration (2024/2025 CSV files available)
- [ ] GeoJSON export for map visualization
- [ ] Route comparison (safest vs shortest vs fastest)
- [ ] Real-time traffic safety data
- [ ] Mobile app integration

## 📄 License

MIT License - See LICENSE file for details

## 👥 Contributors

Built for hackathon project - Bangalore Safe Routing System

---

**Built with ❤️ in Rust** 🦀
