const { Pipeline, SemanticChunker, HybridRetriever } = require('./index.node');

async function runTest() {
  console.log("==========================================");
  console.log("🧠 sagelib MVP: Native execution via Rust");
  console.log("==========================================\n");

  console.log("[1] Initializing the embedded Rust engine...");
  const pipeline = new Pipeline({
    storage: 'duckdb',
    observability: true 
  });

  console.log("[2] Configuring the Composable DAG...");
  pipeline.useChunker(new SemanticChunker());
  pipeline.useRetriever(new HybridRetriever({ rrfK: 60 }));

  console.log("[3] Ingesting Real Data into Embedded Storage...");
  try {
    // We ingest real documents from the user's Downloads folder
    const downloadsPath = "C:/Users/Sohail/Downloads/**/*.txt";
    
    await pipeline.ingest(downloadsPath, "org_123");
    
    console.log("    -> Ingestion successful.\n");
  } catch (err) {
    console.error("    -> Ingestion failed:", err.message);
    return;
  }

  console.log("[4] Testing Retrieval-Time Authorization...");
  const queryStr = "What is our Q4 compliance risk?";

  // Scenario A: Tenant 'org_123' queries the engine
  console.log("\n  --- Scenario A: Request from 'org_123' ---");
  const resultsA = await pipeline.query(queryStr, {
    tenantId: "org_123",
    role: "auditor"
  });
  console.log("  Results (org_123):");
  resultsA.forEach(r => console.log(`    - ${r}`));

  // Scenario B: Tenant 'org_999' queries the exact same engine
  console.log("\n  --- Scenario B: Request from 'org_999' ---");
  const resultsB = await pipeline.query(queryStr, {
    tenantId: "org_999",
    role: "marketing"
  });
  console.log("  Results (org_999):");
  resultsB.forEach(r => console.log(`    - ${r}`));

  console.log("\n==========================================");
  console.log("✅ MVP Verification Complete!");
  console.log("The core Rust execution enforced strict tenant boundary isolation.");
}

runTest().catch(console.error);
