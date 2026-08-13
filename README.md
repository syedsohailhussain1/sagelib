# sagelib

**The Enterprise-Grade Compute Engine for Retrieval-Augmented Generation**

---

## What is sagelib?

`sagelib` is an ultra-fast, "close-to-the-metal" embedded compute engine designed exclusively for production Retrieval-Augmented Generation (RAG). Think of it as the foundational low-level engine for your AI infrastructure.

Instead of treating RAG as a simple linear `parse -> embed -> search` script, `sagelib` treats RAG as a governed decision infrastructure. It provides a robust, pluggable DAG (Directed Acyclic Graph) of composable operators written entirely in Rust, with seamless zero-overhead bindings for Node.js, Python, and Go.

## The Problem it Solves

In 2026, standard vector databases and naive RAG pipelines fail at scale.
1. **Poor Retrieval:** Dense-only vector search misses exact keywords, IDs, and acronyms.
2. **Naive Chunking:** Splitting text by fixed tokens destroys semantic boundaries and context.
3. **No Governance:** Enterprise applications require strict data compliance, audit logs, and tenant-level access control at retrieval time.
4. **Latency and Overhead:** Shuttling data between Python scripts, external chunkers, and remote vector databases introduces massive latency overhead.

## Key Features

- **Hybrid Search and RRF:** Combines dense vectors with sparse (BM25/TF-IDF) search fused via Reciprocal Rank Fusion out of the box.
- **Pluggable Architecture:** A fully composable pipeline graph. Swap out semantic chunkers, query routers, and cross-encoder rerankers effortlessly.
- **Enterprise Security:** Built-in Retrieval-Time Authorization and PII filtering. Filter vectors by tenant ID *before* they ever reach the LLM.
- **Zero-Overhead FFI:** The core engine is written in Rust utilizing SIMD and memory-mapped I/O. Use it directly in Node.js or Python without sacrificing a millisecond of performance.
- **Observability Primitive:** Built-in audit trails and evaluation hooks (Faithfulness, Context Precision) for EU AI Act / NIS2 compliance.

## Installation

**For Node.js / TypeScript:**
```bash
npm install sagelib
```

**For Python:**
```bash
pip install sagelib
```

## Quick Start (Node.js)

```javascript
const { Pipeline, SemanticChunker, HybridRetriever } = require('sagelib');

async function run() {
  // 1. Initialize the embedded Rust engine
  const pipeline = new Pipeline({
    storage: 'duckdb', // or sqlite, pgvector
    observability: true 
  });

  // 2. Configure a composable DAG
  pipeline.useChunker(new SemanticChunker());
  pipeline.useRetriever(new HybridRetriever({ rrfK: 60 }));

  // 3. Ingest Data (Event-Driven or Batch)
  await pipeline.ingest('C:/Downloads/**/*.txt', 'org_123');

  // 4. Query with Retrieval-Time Auth
  const results = await pipeline.query("What is our Q4 compliance risk?", {
    tenantId: "org_123",
    role: "auditor"
  });

  console.log(results);
}

run();
```

## Architecture

`sagelib` operates across a 5-layer platform design:
1. **Compute and Ingestion:** Non-linear, event-driven ingestion supporting semantic and document-aware chunking natively in Rust.
2. **Retrieval Engine:** Hybrid search, cross-encoder reranking, and GraphStore adapters with native TF-IDF mathematical scoring.
3. **Orchestration:** Declarative DAG for adaptive query routing and query transformation (HyDE).
4. **Security:** Native Access Control filters applied deeply within the Rust execution context.
5. **Observability:** Telemetry and audit logging hooks for strict governance.

## Contributing

We welcome contributions! Please see the `CONTRIBUTING.md` to get started.

1. Clone the repo: `git clone https://github.com/syedsohailhussain1/sagelib.git`
2. Install Rust (GNU toolchain recommended for Windows).
3. Run `npm install` and `npm run build`.

## License

MIT License. See `LICENSE` for more information.
