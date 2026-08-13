#![deny(clippy::all)]
#![allow(dead_code)]

#[macro_use]
extern crate napi_derive;

use glob::glob;
use std::fs;
use std::collections::{HashSet};

// --- Mock Document Storage for MVP ---
#[derive(Clone)]
struct Chunk {
  tenant_id: String,
  source: String,
  content: String,
}

// --- Main Pipeline Engine ---

#[napi]
pub struct Pipeline {
  storage_type: String,
  observability_enabled: bool,
  has_chunker: bool,
  has_retriever: bool,
  
  chunks: Arc<Mutex<Vec<Chunk>>>,
}

#[napi]
impl Pipeline {
  #[napi(constructor)]
  pub fn new(options: PipelineOptions) -> Self {
    Pipeline {
      storage_type: options.storage,
      observability_enabled: options.observability,
      has_chunker: false,
      has_retriever: false,
      chunks: Arc::new(Mutex::new(Vec::new())),
    }
  }

  #[napi]
  pub fn use_chunker(&mut self, _chunker: &SemanticChunker) {
    self.has_chunker = true;
  }

  #[napi]
  pub fn use_retriever(&mut self, _retriever: &HybridRetriever) {
    self.has_retriever = true;
  }

  #[napi]
  pub fn ingest(&self, glob_pattern: String, tenant_id: String) -> Result<()> {
    if !self.has_chunker || !self.has_retriever {
      return Err(napi::Error::from_reason("Pipeline requires both a chunker and retriever before ingestion."));
    }

    let mut chunks = self.chunks.lock().unwrap();

    // 1. REAL PARSING: Find all matching files on disk
    for entry in glob(&glob_pattern).expect("Failed to read glob pattern") {
      match entry {
        Ok(path) => {
          let filename = path.display().to_string();
          // We support .txt and .md for this MVP Phase
          if let Ok(content) = fs::read_to_string(&path) {
            
            // 2. REAL CHUNKING: Split into semantic paragraphs
            let split_chunks: Vec<&str> = content.split("\n\n").collect();
            
            for chunk_text in split_chunks {
              let trimmed = chunk_text.trim();
              if !trimmed.is_empty() {
                chunks.push(Chunk {
                  tenant_id: tenant_id.clone(),
                  source: filename.clone(),
                  content: trimmed.to_string(),
                });
              }
            }
          }
        },
        Err(e) => println!("{:?}", e),
      }
    }
    Ok(())
  }

  #[napi]
  pub fn query(&self, query_str: String, options: QueryOptions) -> Result<Vec<String>> {
    let chunks = self.chunks.lock().unwrap();
    
    // 3. REAL TF-IDF HYBRID SEARCH
    let query_terms: Vec<String> = query_str
      .to_lowercase()
      .split_whitespace()
      .map(|s| s.to_string())
      .collect();

    // Calculate Term Frequencies (TF) and Inverse Document Frequency (IDF)
    // TF = (frequency of term in chunk) / (total words in chunk)
    // IDF = log( total_chunks / chunks_containing_term )
    
    let total_chunks = chunks.len() as f64;
    let mut idf_map: HashMap<String, f64> = HashMap::new();

    // Compute IDF
    for term in &query_terms {
      let mut chunks_with_term = 0.0;
      for chunk in chunks.iter() {
        if chunk.tenant_id == options.tenant_id && chunk.content.to_lowercase().contains(term) {
          chunks_with_term += 1.0;
        }
      }
      let idf = if chunks_with_term > 0.0 {
        (total_chunks / chunks_with_term).ln()
      } else {
        0.0
      };
      idf_map.insert(term.clone(), idf);
    }

    // Compute TF-IDF per chunk and rank
    let mut ranked_chunks: Vec<(f64, &Chunk)> = Vec::new();

    for chunk in chunks.iter() {
      // 4. GOVERNANCE: Retrieval-Time Authorization (Strict Boundary)
      if chunk.tenant_id != options.tenant_id {
        continue;
      }

      let chunk_words: Vec<&str> = chunk.content.split_whitespace().collect();
      let total_words = chunk_words.len() as f64;
      if total_words == 0.0 { continue; }

      let chunk_lower = chunk.content.to_lowercase();
      let mut total_score = 0.0;

      for term in &query_terms {
        // Simple term counting
        let term_count = chunk_lower.matches(term).count() as f64;
        let tf = term_count / total_words;
        let idf = idf_map.get(term).unwrap_or(&0.0);
        total_score += tf * idf;
      }

      if total_score > 0.0 {
        ranked_chunks.push((total_score, chunk));
      }
    }

    // Sort by descending TF-IDF score
    ranked_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Extract top 3 results
    let mut results = Vec::new();
    for (score, chunk) in ranked_chunks.iter().take(3) {
      results.push(format!("[Score: {:.4}] (Source: {}) {}", score, chunk.source, chunk.content));
    }

    if self.observability_enabled {
      println!("[sagelib-telemetry] Real TF-IDF query executed for tenant '{}'. Returned top {} chunks.", options.tenant_id, results.len());
    }

    Ok(results)
  }
}
