const assert = require('node:assert');
const path = require('node:path');
const { Pipeline, SemanticChunker, HybridRetriever } = require('./index.node');

async function runTestSuite() {
  console.log("==========================================");
  console.log("🧪 Running sagelib Automated Test Suite");
  console.log("==========================================\n");

  let passed = 0;
  let failed = 0;

  function test(name, fn) {
    try {
      fn();
      console.log(`  ✓ ${name}`);
      passed++;
    } catch (err) {
      console.error(`  ✗ ${name}`);
      console.error(`    ${err.message}`);
      failed++;
    }
  }

  async function asyncTest(name, fn) {
    try {
      await fn();
      console.log(`  ✓ ${name}`);
      passed++;
    } catch (err) {
      console.error(`  ✗ ${name}`);
      console.error(`    ${err.message}`);
      failed++;
    }
  }

  // Test 1: Instantiate operators
  test("SemanticChunker instantiation", () => {
    const chunker = new SemanticChunker();
    assert.ok(chunker, "SemanticChunker should instantiate");
  });

  test("HybridRetriever instantiation", () => {
    const retriever = new HybridRetriever({ rrfK: 60 });
    assert.ok(retriever, "HybridRetriever should instantiate");
  });

  // Test 2: Unconfigured pipeline throws error on ingestion
  await asyncTest("Pipeline requires chunker and retriever before ingest", async () => {
    const unconfigured = new Pipeline({ storage: "local", observability: false });
    let threw = false;
    try {
      await unconfigured.ingest("fixtures/**/*.txt", "org_test");
    } catch (err) {
      threw = true;
      assert.match(err.message, /requires both a chunker and retriever/i);
    }
    assert.ok(threw, "Ingestion should fail without configured operators");
  });

  // Test 3: End-to-end ingestion and retrieval
  await asyncTest("End-to-end ingestion and search ranking", async () => {
    const pipeline = new Pipeline({ storage: "duckdb", observability: false });
    pipeline.useChunker(new SemanticChunker());
    pipeline.useRetriever(new HybridRetriever({ rrfK: 60 }));

    const fixturesGlob = path.join(__dirname, "fixtures", "*.txt").replace(/\\/g, "/");
    await pipeline.ingest(fixturesGlob, "tenant_alpha");

    const results = await pipeline.query("Q4 compliance risk", {
      tenantId: "tenant_alpha",
      role: "auditor"
    });

    assert.ok(Array.isArray(results), "Results should be an array");
    assert.ok(results.length > 0, "Should return matching chunks");
    assert.ok(results[0].includes("Compliance Risk Report") || results[0].includes("Q4"), "Top result should be relevant chunk");
  });

  // Test 4: Strict Multi-Tenant Isolation
  await asyncTest("Multi-tenant boundary isolation", async () => {
    const pipeline = new Pipeline({ storage: "duckdb", observability: false });
    pipeline.useChunker(new SemanticChunker());
    pipeline.useRetriever(new HybridRetriever({ rrfK: 60 }));

    const fixturesGlob = path.join(__dirname, "fixtures", "*.txt").replace(/\\/g, "/");
    await pipeline.ingest(fixturesGlob, "tenant_alpha");

    // Query with unauthorized tenant ID
    const unauthorizedResults = await pipeline.query("Q4 compliance risk", {
      tenantId: "tenant_beta",
      role: "auditor"
    });

    assert.strictEqual(unauthorizedResults.length, 0, "Unauthorized tenant must receive 0 results");
  });

  // Test 5: Unmatched query returns 0 results cleanly
  await asyncTest("Unmatched keywords query returns empty list", async () => {
    const pipeline = new Pipeline({ storage: "duckdb", observability: false });
    pipeline.useChunker(new SemanticChunker());
    pipeline.useRetriever(new HybridRetriever({ rrfK: 60 }));

    const fixturesGlob = path.join(__dirname, "fixtures", "*.txt").replace(/\\/g, "/");
    await pipeline.ingest(fixturesGlob, "tenant_alpha");

    const results = await pipeline.query("nonexistent_random_token_xyz", {
      tenantId: "tenant_alpha",
      role: "viewer"
    });

    assert.strictEqual(results.length, 0, "Query with non-matching terms should return 0 results");
  });

  console.log("\n==========================================");
  console.log(`Summary: ${passed} passed, ${failed} failed`);
  console.log("==========================================");

  if (failed > 0) {
    process.exit(1);
  }
}

runTestSuite().catch(err => {
  console.error("Test Suite crashed:", err);
  process.exit(1);
});
