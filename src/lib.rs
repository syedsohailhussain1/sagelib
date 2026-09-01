use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use glob::glob;

// ==========================================
// 🧠 Core Shared Engine with Inverted Index
// ==========================================

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub tenant_id: String,
    pub source: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct Posting {
    pub chunk_idx: usize,
    pub tf: f64,
}

#[derive(Default, Clone, Debug)]
pub struct TenantIndex {
    pub chunk_count: usize,
    pub inverted_index: HashMap<String, Vec<Posting>>,
    pub chunk_lengths: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct CoreSemanticChunker {
    pub config: String,
}

impl CoreSemanticChunker {
    pub fn new() -> Self {
        Self {
            config: "default".to_string(),
        }
    }

    pub fn chunk(&self, content: &str) -> Vec<String> {
        content
            .split("\n\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl Default for CoreSemanticChunker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct CoreHybridRetriever {
    pub rrf_k: i32,
}

impl CoreHybridRetriever {
    pub fn new(rrf_k: i32) -> Self {
        Self { rrf_k }
    }
}

impl Default for CoreHybridRetriever {
    fn default() -> Self {
        Self::new(60)
    }
}

pub struct CorePipeline {
    pub storage_type: String,
    pub observability_enabled: bool,
    pub has_chunker: bool,
    pub has_retriever: bool,
    pub chunks: Arc<Mutex<Vec<Chunk>>>,
    pub tenant_indexes: Arc<Mutex<HashMap<String, TenantIndex>>>,
}

impl CorePipeline {
    pub fn new(storage: String, observability: bool) -> Self {
        Self {
            storage_type: storage,
            observability_enabled: observability,
            has_chunker: false,
            has_retriever: false,
            chunks: Arc::new(Mutex::new(Vec::new())),
            tenant_indexes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn use_chunker(&mut self, _chunker: &CoreSemanticChunker) {
        self.has_chunker = true;
    }

    pub fn use_retriever(&mut self, _retriever: &CoreHybridRetriever) {
        self.has_retriever = true;
    }

    pub fn ingest_text(&self, source: &str, content: &str, tenant_id: &str) -> Result<usize, String> {
        if !self.has_chunker || !self.has_retriever {
            return Err("Pipeline requires both a chunker and retriever before ingestion.".to_string());
        }

        let chunker = CoreSemanticChunker::new();
        let split_chunks = chunker.chunk(content);
        let count = split_chunks.len();

        let mut chunks = self.chunks.lock().map_err(|e| e.to_string())?;
        let mut indexes = self.tenant_indexes.lock().map_err(|e| e.to_string())?;
        let tenant_index = indexes.entry(tenant_id.to_string()).or_default();

        for chunk_text in split_chunks {
            let global_idx = chunks.len();
            chunks.push(Chunk {
                tenant_id: tenant_id.to_string(),
                source: source.to_string(),
                content: chunk_text.clone(),
            });

            // Tokenize words and clean punctuation
            let words: Vec<String> = chunk_text
                .split_whitespace()
                .map(|w| {
                    w.chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                        .to_lowercase()
                })
                .filter(|w| !w.is_empty())
                .collect();

            let total_words = words.len();
            tenant_index.chunk_count += 1;
            tenant_index.chunk_lengths.push(total_words);

            if total_words == 0 {
                continue;
            }

            // Calculate term frequencies
            let mut term_counts: HashMap<String, usize> = HashMap::new();
            for word in words {
                *term_counts.entry(word).or_insert(0) += 1;
            }

            // Populate Inverted Index
            for (term, term_count) in term_counts {
                let tf = (term_count as f64) / (total_words as f64);
                tenant_index
                    .inverted_index
                    .entry(term)
                    .or_default()
                    .push(Posting {
                        chunk_idx: global_idx,
                        tf,
                    });
            }
        }

        Ok(count)
    }

    pub fn ingest_glob(&self, glob_pattern: &str, tenant_id: &str) -> Result<usize, String> {
        if !self.has_chunker || !self.has_retriever {
            return Err("Pipeline requires both a chunker and retriever before ingestion.".to_string());
        }

        let entries = glob(glob_pattern).map_err(|e| e.to_string())?;
        let mut total_chunks = 0;

        for entry in entries {
            match entry {
                Ok(path) => {
                    let filename = path.display().to_string();
                    if let Ok(content) = fs::read_to_string(&path) {
                        let count = self.ingest_text(&filename, &content, tenant_id)?;
                        total_chunks += count;
                    }
                }
                Err(e) => {
                    if self.observability_enabled {
                        eprintln!("[sagelib-telemetry] Glob error: {:?}", e);
                    }
                }
            }
        }

        Ok(total_chunks)
    }

    pub fn query(&self, query_str: &str, tenant_id: &str) -> Result<Vec<String>, String> {
        // Clean and tokenize query terms
        let query_terms: Vec<String> = query_str
            .split_whitespace()
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect();

        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        let indexes = self.tenant_indexes.lock().map_err(|e| e.to_string())?;
        let tenant_index = match indexes.get(tenant_id) {
            Some(idx) if idx.chunk_count > 0 => idx,
            _ => return Ok(Vec::new()),
        };

        let total_tenant_chunks = tenant_index.chunk_count as f64;
        let mut chunk_scores: HashMap<usize, f64> = HashMap::new();

        // Instantaneous Inverted Index lookup: O(1) hash table lookup per query term
        for term in &query_terms {
            if let Some(postings) = tenant_index.inverted_index.get(term) {
                let df = postings.len() as f64;
                let idf = (total_tenant_chunks / df).ln() + 1.0;
                for posting in postings {
                    *chunk_scores.entry(posting.chunk_idx).or_insert(0.0) += posting.tf * idf;
                }
            }
        }

        if chunk_scores.is_empty() {
            return Ok(Vec::new());
        }

        // Rank scored documents
        let mut ranked: Vec<(f64, usize)> = chunk_scores
            .into_iter()
            .map(|(idx, score)| (score, idx))
            .collect();

        ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let chunks = self.chunks.lock().map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        for (score, chunk_idx) in ranked.into_iter().take(3) {
            if let Some(chunk) = chunks.get(chunk_idx) {
                results.push(format!(
                    "[Score: {:.4}] (Source: {}) {}",
                    score, chunk.source, chunk.content
                ));
            }
        }

        if self.observability_enabled {
            println!(
                "[sagelib-telemetry] Executed query for tenant '{}'. Found {} chunks.",
                tenant_id,
                results.len()
            );
        }

        Ok(results)
    }
}

// ==========================================
// 🌐 Node.js N-API Bindings
// ==========================================

#[cfg(feature = "node")]
pub mod node {
    use super::*;
    use napi::Result;
    use napi_derive::napi;

    #[napi(object)]
    pub struct PipelineOptions {
        pub storage: String,
        pub observability: bool,
    }

    #[napi(object)]
    pub struct QueryOptions {
        pub tenant_id: String,
        pub role: String,
    }

    #[napi(object)]
    pub struct HybridRetrieverOptions {
        pub rrf_k: i32,
    }

    #[napi]
    pub struct SemanticChunker {
        inner: CoreSemanticChunker,
    }

    #[napi]
    impl SemanticChunker {
        #[napi(constructor)]
        pub fn new() -> Self {
            Self {
                inner: CoreSemanticChunker::new(),
            }
        }
    }

    #[napi]
    pub struct HybridRetriever {
        inner: CoreHybridRetriever,
    }

    #[napi]
    impl HybridRetriever {
        #[napi(constructor)]
        pub fn new(options: HybridRetrieverOptions) -> Self {
            Self {
                inner: CoreHybridRetriever::new(options.rrf_k),
            }
        }
    }

    #[napi]
    pub struct Pipeline {
        inner: CorePipeline,
    }

    #[napi]
    impl Pipeline {
        #[napi(constructor)]
        pub fn new(options: PipelineOptions) -> Self {
            Self {
                inner: CorePipeline::new(options.storage, options.observability),
            }
        }

        #[napi]
        pub fn use_chunker(&mut self, chunker: &SemanticChunker) {
            self.inner.use_chunker(&chunker.inner);
        }

        #[napi]
        pub fn use_retriever(&mut self, retriever: &HybridRetriever) {
            self.inner.use_retriever(&retriever.inner);
        }

        #[napi]
        pub fn ingest(&mut self, glob_pattern: String, tenant_id: String) -> Result<()> {
            self.inner
                .ingest_glob(&glob_pattern, &tenant_id)
                .map_err(|e| napi::Error::from_reason(e))?;
            Ok(())
        }

        #[napi]
        pub fn query(&self, query_str: String, options: QueryOptions) -> Result<Vec<String>> {
            self.inner
                .query(&query_str, &options.tenant_id)
                .map_err(|e| napi::Error::from_reason(e))
        }
    }
}

// ==========================================
// 🐍 Python PyO3 Bindings
// ==========================================

#[cfg(feature = "python")]
pub mod python {
    use super::*;
    use pyo3::prelude::*;

    #[pyclass]
    pub struct SemanticChunker {
        inner: CoreSemanticChunker,
    }

    #[pymethods]
    impl SemanticChunker {
        #[new]
        pub fn new() -> Self {
            Self {
                inner: CoreSemanticChunker::new(),
            }
        }
    }

    #[pyclass]
    pub struct HybridRetriever {
        inner: CoreHybridRetriever,
    }

    #[pymethods]
    impl HybridRetriever {
        #[new]
        pub fn new(rrf_k: i32) -> Self {
            Self {
                inner: CoreHybridRetriever::new(rrf_k),
            }
        }
    }

    #[pyclass]
    pub struct Pipeline {
        inner: CorePipeline,
    }

    #[pymethods]
    impl Pipeline {
        #[new]
        pub fn new(storage: String, observability: bool) -> Self {
            Self {
                inner: CorePipeline::new(storage, observability),
            }
        }

        pub fn use_chunker(&mut self, chunker: &SemanticChunker) {
            self.inner.use_chunker(&chunker.inner);
        }

        pub fn use_retriever(&mut self, retriever: &HybridRetriever) {
            self.inner.use_retriever(&retriever.inner);
        }

        pub fn ingest(&mut self, glob_pattern: String, tenant_id: String) -> PyResult<()> {
            self.inner
                .ingest_glob(&glob_pattern, &tenant_id)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
            Ok(())
        }

        pub fn query(&self, query_str: String, tenant_id: String) -> PyResult<Vec<String>> {
            self.inner
                .query(&query_str, &tenant_id)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
        }
    }

    #[pymodule]
    fn sagelib(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<SemanticChunker>()?;
        m.add_class::<HybridRetriever>()?;
        m.add_class::<Pipeline>()?;
        Ok(())
    }
}

// ==========================================
// 🧪 Automated Unit Tests
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_requires_chunker_and_retriever() {
        let pipeline = CorePipeline::new("local".to_string(), false);
        let res = pipeline.ingest_text("doc1.txt", "Some content", "tenant1");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("requires both a chunker and retriever"));
    }

    #[test]
    fn test_semantic_chunker_splits_paragraphs() {
        let chunker = CoreSemanticChunker::new();
        let content = "First paragraph with details.\n\nSecond paragraph with more info.\n\n\nThird paragraph.";
        let chunks = chunker.chunk(content);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "First paragraph with details.");
        assert_eq!(chunks[1], "Second paragraph with more info.");
        assert_eq!(chunks[2], "Third paragraph.");
    }

    #[test]
    fn test_multi_tenant_boundary_isolation() {
        let mut pipeline = CorePipeline::new("local".to_string(), false);
        let chunker = CoreSemanticChunker::new();
        let retriever = CoreHybridRetriever::new(60);
        pipeline.use_chunker(&chunker);
        pipeline.use_retriever(&retriever);

        // Ingest for Tenant A
        pipeline
            .ingest_text("secret_a.txt", "Tenant A highly confidential financial risk data.", "tenant_A")
            .unwrap();

        // Ingest for Tenant B
        pipeline
            .ingest_text("secret_b.txt", "Tenant B confidential marketing launch plans.", "tenant_B")
            .unwrap();

        // Tenant A queries for risk
        let results_a = pipeline.query("financial risk", "tenant_A").unwrap();
        assert!(!results_a.is_empty());
        assert!(results_a[0].contains("Tenant A"));
        assert!(!results_a[0].contains("Tenant B"));

        // Tenant B queries for risk -> should get 0 results (isolated)
        let results_b_query_a = pipeline.query("financial risk", "tenant_B").unwrap();
        assert_eq!(results_b_query_a.len(), 0);

        // Tenant C (unauthorized) queries -> 0 results
        let results_c = pipeline.query("financial risk", "tenant_C").unwrap();
        assert_eq!(results_c.len(), 0);
    }

    #[test]
    fn test_tfidf_ranking_relevance() {
        let mut pipeline = CorePipeline::new("local".to_string(), false);
        let chunker = CoreSemanticChunker::new();
        let retriever = CoreHybridRetriever::new(60);
        pipeline.use_chunker(&chunker);
        pipeline.use_retriever(&retriever);

        let doc = "Artificial intelligence and machine learning models.\n\nDeep learning neural network architectures for computer vision.\n\nUnrelated cooking recipe for apple pie.";
        pipeline.ingest_text("tech.txt", doc, "org_1").unwrap();

        let results = pipeline.query("machine learning models", "org_1").unwrap();
        assert!(!results.is_empty());
        // Top result should be the first chunk
        assert!(results[0].contains("Artificial intelligence and machine learning"));
    }

    #[test]
    fn test_empty_query_handling() {
        let mut pipeline = CorePipeline::new("local".to_string(), false);
        let chunker = CoreSemanticChunker::new();
        let retriever = CoreHybridRetriever::new(60);
        pipeline.use_chunker(&chunker);
        pipeline.use_retriever(&retriever);

        pipeline.ingest_text("test.txt", "Hello world", "tenant_x").unwrap();
        let results = pipeline.query("", "tenant_x").unwrap();
        assert_eq!(results.len(), 0);
    }
}
