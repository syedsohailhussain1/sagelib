import sagelib

print("Initializing Pipeline...")
pipeline = sagelib.Pipeline("local", True)

print("Setting up chunker and retriever...")
chunker = sagelib.SemanticChunker()
retriever = sagelib.HybridRetriever(60)

pipeline.use_chunker(chunker)
pipeline.use_retriever(retriever)

print("Ingesting test files...")
# Adjust glob pattern according to the user's Downloads path where docs live
pipeline.ingest("C:/Users/Sohail/Downloads/*.txt", "test-tenant")

print("Executing query...")
results = pipeline.query("machine learning", "test-tenant")
for r in results:
    print(r)
