import os
import sagelib

print("Initializing Pipeline...")
pipeline = sagelib.Pipeline("local", True)

print("Setting up chunker and retriever...")
chunker = sagelib.SemanticChunker()
retriever = sagelib.HybridRetriever(60)

pipeline.use_chunker(chunker)
pipeline.use_retriever(retriever)

print("Ingesting test files...")
fixtures_glob = os.path.join(os.path.dirname(__file__), "fixtures", "*.txt").replace("\\", "/")
pipeline.ingest(fixtures_glob, "test-tenant")

print("Executing query...")
results = pipeline.query("machine learning", "test-tenant")
for r in results:
    print(r)
