# sagelib

**Fast, Embedded In-Process Search & Multi-Tenant Retrieval Engine for RAG**

---

## What is sagelib?

`sagelib` is an embedded, in-process compute engine designed for Retrieval-Augmented Generation (RAG) workflows. Written in **Rust** with native bindings for **Node.js** (N-API) and **Python** (PyO3), `sagelib` runs close to the metal with zero network serialization overhead and no external database dependencies.

Instead of deploying and managing heavyweight external vector databases for local-first, desktop, or edge AI applications, `sagelib` gives you a fast, lightweight, and memory-safe retrieval engine that runs entirely within your application process.

---

## Why sagelib?

| Problem in Standard RAG | How `sagelib` Solves It |
| :--- | :--- |
| **Network & Serialization Lag:** Shuttling documents and embeddings across external microservices and databases introduces latency. | **In-Process Rust Engine:** Runs natively inside Node.js or Python via direct FFI with zero JSON/network serialization overhead. |
| **Tenant Data Bleed:** Filtering multi-tenant data in prompt templates or post-retrieval is vulnerable to leaks. | **Retrieval-Time Isolation:** Hard boundary isolation deeply enforced in Rust; unauthorized tenants never receive data. |
| **Keyword & Acronym Misses:** Dense-only vector embeddings struggle with exact product IDs, acronyms, and compliance codes. | **Native Sparse TF-IDF Engine:** Exact token frequency and inverse document frequency scoring for precise matching. |
| **Heavy Infrastructure Footprint:** Managing Docker containers or cloud vector database clusters for simple local or desktop AI tools. | **Zero Dependencies:** Pure embedded library that initializes instantly with zero configuration. |

---

## Key Features

- **Blazing Fast In-Process Execution:** Core indexing, chunking, and search math are implemented in compiled Rust.
- **Strict Multi-Tenant Isolation:** Documents are bound to immutable tenant identifiers during ingestion and strictly filtered before results are returned.
- **Semantic Paragraph Chunking:** Automatically segments documents along paragraph boundaries, preserving semantic cohesion.
- **Sparse TF-IDF Retrieval:** Calculates term frequency and inverse document frequency across tenant-specific document collections.
- **Dual Runtime Support:** First-class bindings for both Node.js / TypeScript and Python.
- **Event / Glob Document Ingestion:** Recursively discovers and ingests `.txt` and `.md` document collections.

---

## Getting Started (Build from Source)

`sagelib` is currently available directly from source.

### 1. Clone the Repository
```bash
git clone https://github.com/syedsohailhussain1/sagelib.git
cd sagelib
```

### 2. For Node.js / TypeScript
```bash
# Install development dependencies
npm install

# Run automated test suite
npm test

# Run example demonstration
node index.js
```

### 3. For Python
```bash
# Install maturin build tool
pip install maturin

# Build and test python bindings
maturin develop --features python
python test.py
```

---

## Quick Start

### Node.js Example

```javascript
const path = require('node:path');
// Import native binary from local build
const { Pipeline, SemanticChunker, HybridRetriever } = require('./index.node');

async function main() {
  // 1. Initialize pipeline
  const pipeline = new Pipeline({
    storage: 'local',
    observability: true
  });

  // 2. Attach composable operators
  pipeline.useChunker(new SemanticChunker());
  pipeline.useRetriever(new HybridRetriever({ rrfK: 60 }));

  // 3. Ingest documents for a specific tenant
  const fixturesPath = path.join(__dirname, 'fixtures', '*.txt').replace(/\\/g, '/');
  await pipeline.ingest(fixturesPath, 'tenant_123');

  // 4. Query with strict tenant-level authorization
  const results = await pipeline.query('compliance risk', {
    tenantId: 'tenant_123',
    role: 'auditor'
  });

  console.log('Results:', results);
}

main();
```

### Python Example

```python
import os
import sagelib

# 1. Initialize pipeline
pipeline = sagelib.Pipeline("local", True)

# 2. Attach chunker and retriever
chunker = sagelib.SemanticChunker()
retriever = sagelib.HybridRetriever(60)
pipeline.use_chunker(chunker)
pipeline.use_retriever(retriever)

# 3. Ingest documents for a specific tenant
fixtures_glob = os.path.join(os.path.dirname(__file__), "fixtures", "*.txt").replace("\\", "/")
pipeline.ingest(fixtures_glob, "tenant_123")

# 4. Query engine
results = pipeline.query("compliance risk", "tenant_123")
for r in results:
    print(r)
```

---

## Architecture

`sagelib` is structured into three layers:

```
┌─────────────────────────────────────────────────────────┐
│              Host Applications (Node.js / Python)       │
├───────────────────────────┬─────────────────────────────┤
│   Node.js N-API Layer     │     Python PyO3 Layer       │
├───────────────────────────┴─────────────────────────────┤
│                   Core Rust Engine                      │
│  - CorePipeline (Coordinator & State)                   │
│  - CoreSemanticChunker (Paragraph splitting & parsing)  │
│  - Sparse TF-IDF Ranker & Tenant Isolation Filters      │
│  - Thread-Safe Shared Document Chunks                   │
└─────────────────────────────────────────────────────────┘
```

---

## Development & Testing

### Running Tests

**Automated Node.js Test Suite:**
```bash
npm test
```

**Rust Core Unit Tests:**
```bash
cargo test --no-default-features
```

**Run Demo Scripts:**
```bash
node index.js
python test.py
```

---

## Roadmap

- [ ] **Package Distribution**: Official publishing to npm and PyPI registries.
- [ ] **Persistent Storage**: Embedded DuckDB / SQLite disk persistence.
- [ ] **Full BM25 Scoring**: Add document length normalization ($k_1$, $b$).
- [ ] **Dense Vector Embeddings**: SIMD-accelerated cosine similarity and HNSW index.
- [ ] **Reciprocal Rank Fusion (RRF)**: Combining sparse keyword and dense embedding ranking lists.
- [ ] **Go Bindings**: CGO/FFI bindings for Go applications.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
