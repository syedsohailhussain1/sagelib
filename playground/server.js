const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
const { performance } = require('node:perf_hooks');
const { Pipeline, SemanticChunker, HybridRetriever } = require('../index.node');

const PORT = 3000;
const pipeline = new Pipeline({ storage: 'local', observability: true });
pipeline.useChunker(new SemanticChunker());
pipeline.useRetriever(new HybridRetriever({ rrfK: 60 }));

// Pre-load data
async function preloadData() {
  console.log('Ingesting demo datasets into sagelib...');
  
  // Ingest fixtures
  const fixturesPath = path.join(__dirname, '..', 'fixtures', '*.txt').replace(/\\/g, '/');
  await pipeline.ingest(fixturesPath, 'org_123');

  // Ingest sample tenant data if available
  const dataDir = path.join(__dirname, '..', 'benchmark', 'data');
  const tenants = ['tenant_finance', 'tenant_engineering', 'tenant_healthcare', 'tenant_legal', 'tenant_cybersecurity'];
  
  for (const t of tenants) {
    const tGlob = path.join(dataDir, t, '*.txt').replace(/\\/g, '/');
    try {
      await pipeline.ingest(tGlob, t);
    } catch (e) {
      // Benchmark data may not be prepared yet
    }
  }
  console.log('Data ingestion complete!');
}

const server = http.createServer(async (req, res) => {
  const parsedUrl = new URL(req.url, `http://${req.headers.host}`);
  
  // API: Search
  if (parsedUrl.pathname === '/api/search') {
    const q = parsedUrl.searchParams.get('q') || '';
    const tenant = parsedUrl.searchParams.get('tenant') || 'org_123';
    
    const t0 = performance.now();
    let results = [];
    try {
      results = await pipeline.query(q, { tenantId: tenant, role: 'user' });
    } catch (err) {
      console.error(err);
    }
    const t1 = performance.now();
    const latencyMicros = ((t1 - t0) * 1000).toFixed(1);
    const latencyMs = (t1 - t0).toFixed(3);

    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      query: q,
      tenant,
      latencyMicros,
      latencyMs,
      count: results.length,
      results
    }));
    return;
  }

  // API: Ingest text directly
  if (parsedUrl.pathname === '/api/ingest' && req.method === 'POST') {
    let body = '';
    req.on('data', chunk => { body += chunk; });
    req.on('end', async () => {
      try {
        const data = JSON.parse(body);
        const tmpPath = path.join(__dirname, `upload_${Date.now()}.txt`);
        fs.writeFileSync(tmpPath, data.content, 'utf-8');
        await pipeline.ingest(tmpPath.replace(/\\/g, '/'), data.tenantId || 'custom_tenant');
        try { fs.unlinkSync(tmpPath); } catch (_) {}
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ success: true, message: 'Document ingested successfully' }));
      } catch (err) {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ success: false, error: err.message }));
      }
    });
    return;
  }

  // Serve static HTML UI
  if (parsedUrl.pathname === '/' || parsedUrl.pathname === '/index.html') {
    const htmlPath = path.join(__dirname, 'public', 'index.html');
    if (fs.existsSync(htmlPath)) {
      res.writeHead(200, { 'Content-Type': 'text/html' });
      res.end(fs.readFileSync(htmlPath, 'utf-8'));
      return;
    }
  }

  res.writeHead(404, { 'Content-Type': 'text/plain' });
  res.end('Not Found');
});

preloadData().then(() => {
  server.listen(PORT, () => {
    console.log(`\n✨ sagelib Visual Interactive Playground running at: http://localhost:${PORT}`);
  });
});
