# 📚 Concurrent Dictionary API Server

A high-performance, concurrent REST API server built in Rust for generating, managing, and serving dictionaries using data from the [Random Words API](https://random-words-api.vercel.app/word).

---
## Run application
```rust
cargo run
```

## Build The application
```rust
cargo build
```

---

## 🚀 Features

- ✅ **Generate dictionaries** with custom names and word counts
- ⚙️ Handles **concurrent client requests** efficiently
- 🔄 Background processing with **job status tracking**
- 📁 **Download generated dictionaries**
- 📊 **Statistics per dictionary** (word count by starting letter)
- 🗑️ **Delete** dictionaries by name

---

## 🛠️ Tech Stack

- **Rust**
- [`actix-web`](https://actix.rs/)
- `tokio` for async concurrency
- `DashMap` for concurrent in-memory state
- `env_logger` for structured logging
- `serde` / `serde_json` for data serialization
- `reqwest` for outbound HTTP requests
- File-based storage for dictionaries

---

## 📂 API Overview

### 1. **Generate Dictionary**
- `POST /api/v1/dict/generate`
- **Body**:
```json
{
  "dict_name": "animals",
  "word_count": 100
}
```

### 2. **Check Dictionary Status**
- `GET /api/v1/dict/{dict_name}/status`
- **Body**:
```json
{
  "message": "Dictionary exist",
  "status": "Completed"
}
```

### 3. **Delete a Dictionary**
- `DELETE /api/v1/dict/{dict_name}`
- **Response**:
```json
{
  "message": "deleted successfully",
  "status": true
}
```

### 4. **Get Statistcis of a Dictionary**
- `GET /api/v1/dict/{dict_name}/statistics`
- **Response**:
```json
{
  "message": "Dictionary exist",
  "stats": {
    "A": 4,
    "B": 4,
    "C": 4,
    "D": 3,
    "E": 5,
    "F": 5,
    "G": 6,
    "H": 6
  }
}
```

### 5. **Download a Dictionary**
- `GET /api/v1/dict/{dict_name}/download`
- **Response**:
```text
Advocaat: Atfokat, Liqueur containing rum and raw eggs  
Apatetic: Apatetik, Of an animal's coloration or markings  
Aphemia: Afemia, Loss of ability to produce articulate speech  
Aphrodisiomania: Afrotiksiomania, Abnormal sexual interest  
Basilicon: Basilikon, Kind of ointment  
Batraquomancy: Batrakuomans, Divination using frogs  
Belvedere: Belfetere, Raised covered terrace or pavilion  
Biognosy: Bioknos, General study or theory of life  
Caboched: Kabokshet, Heraldic animal shown in full face with no neck or body  
Cartomancy: Kartomans, Telling fortunes using playing cards  
Catastasis: Katastasis, Part of drama with highest action; climax   
Chessel: Shesel, Cheese-mould  
Daboya: Taboya, Large Indian viper  
Dealate: Tealate, Insect divested of its wings
```

