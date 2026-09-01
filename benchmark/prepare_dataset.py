import os
import urllib.request

BENCHMARK_DIR = os.path.join(os.path.dirname(__file__), "data")
os.makedirs(BENCHMARK_DIR, exist_ok=True)

# Public datasets to download
SOURCES = [
    ("gutenberg_alice.txt", "https://www.gutenberg.org/files/11/11-0.txt"),
    ("gutenberg_sherlock.txt", "https://www.gutenberg.org/files/1661/1661-0.txt"),
    ("rfc_http.txt", "https://www.ietf.org/rfc/rfc2616.txt"),
]

print("1. Downloading public baseline datasets...")
for filename, url in SOURCES:
    path = os.path.join(BENCHMARK_DIR, filename)
    if not os.path.exists(path):
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req, timeout=10) as resp:
                content = resp.read().decode('utf-8', errors='ignore')
                with open(path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"   Downloaded {filename} ({len(content)} chars)")
        except Exception as e:
            print(f"   Download failed for {filename}: {e}")
    else:
        print(f"   Found cached {filename}")

# Multi-tenant enterprise dataset
TENANTS = [
    "tenant_finance",
    "tenant_engineering",
    "tenant_healthcare",
    "tenant_legal",
    "tenant_cybersecurity",
    "tenant_operations",
    "tenant_marketing",
    "tenant_human_resources",
    "tenant_compliance",
    "tenant_procurement"
]

TOPICS = {
    "tenant_finance": [
        "Q4 Financial Risk and Treasury Hedging Strategies against currency fluctuation.",
        "Internal Audit Report regarding Sarbanes-Oxley Section 404 internal control compliance.",
        "Mergers and Acquisitions Valuation Model utilizing discounted cash flow and EBITDA multiples.",
        "Revenue Recognition Guidelines under ASC 606 for subscription SaaS contracts."
    ],
    "tenant_engineering": [
        "High-performance distributed database indexing using LSM trees and memory-mapped files.",
        "Microservices architecture migration from legacy monolith to Kubernetes cluster.",
        "Rust SIMD acceleration techniques for vectorized floating-point operations.",
        "Zero-allocation streaming data pipelines with async Tokio runtimes."
    ],
    "tenant_healthcare": [
        "Clinical trial Phase III double-blind randomized controlled study efficacy results.",
        "HIPAA Privacy Rule compliance for protected health information de-identification.",
        "Electronic Health Record interoperability standards using HL7 FHIR protocols.",
        "Adverse drug event reporting and pharmacovigilance surveillance procedures."
    ],
    "tenant_legal": [
        "Mutual Non-Disclosure Agreement confidentiality terms and trade secret definitions.",
        "Cross-border personal data transfer compliance under GDPR Chapter V Standard Contractual Clauses.",
        "Patent infringement liability and intellectual property indemnification clauses.",
        "Executive employment agreement non-compete enforceability guidelines."
    ],
    "tenant_cybersecurity": [
        "SOC 2 Type II compliance audit framework covering security, availability, and confidentiality.",
        "Zero-trust network architecture with mutual TLS authentication and micro-segmentation.",
        "Vulnerability assessment and penetration testing remediation roadmap.",
        "Incident response playbook for distributed denial of service and ransomware mitigation."
    ],
    "tenant_operations": [
        "Supply chain logistics optimization reducing transit lead times across distribution hubs.",
        "Warehouse inventory management system automated picking algorithms.",
        "Vendor service level agreement monitoring and penalty calculations.",
        "Facilities physical security access controls and emergency protocols."
    ],
    "tenant_marketing": [
        "Global brand campaign performance metrics across organic search and paid acquisition.",
        "Customer lifetime value optimization through predictive churn modeling.",
        "Product launch go-to-market messaging and competitive positioning matrix.",
        "Content marketing strategy and developer community advocacy roadmap."
    ],
    "tenant_human_resources": [
        "Global talent acquisition compensation bands and equity grant frameworks.",
        "Annual employee performance review calibration and promotion criteria.",
        "Remote work policy compliance and international tax residency guidelines.",
        "Employee health benefits and wellness program participation rates."
    ],
    "tenant_compliance": [
        "Anti-Money Laundering AML transaction monitoring thresholds and suspicious activity reporting.",
        "Corporate governance code of conduct policies and whistleblower protection standards.",
        "Export control regulations and international sanctions screening procedures.",
        "Environmental Social and Governance ESG carbon footprint reporting mandates."
    ],
    "tenant_procurement": [
        "Enterprise software vendor master service agreement standard pricing tier negotiation.",
        "Hardware procurement supply chain redundancy protocols and purchase order approvals.",
        "Third-party vendor risk assessment matrix and security assurance audits.",
        "Strategic vendor quarterly business review scorecards and performance metrics."
    ]
}

print("2. Synthesizing large multi-tenant corpus (25,000 documents, 75,000 paragraphs)...")
DOCS_PER_TENANT = 2500  # 10 tenants * 2500 docs = 25,000 documents
total_generated = 0
total_bytes = 0

for tenant in TENANTS:
    tenant_dir = os.path.join(BENCHMARK_DIR, tenant)
    os.makedirs(tenant_dir, exist_ok=True)
    topics = TOPICS[tenant]
    
    for i in range(DOCS_PER_TENANT):
        topic = topics[i % len(topics)]
        doc_filename = f"doc_{i:04d}.txt"
        doc_path = os.path.join(tenant_dir, doc_filename)
        
        # Build multi-paragraph realistic document
        p1 = f"{topic} Document ID: {tenant.upper()}-DOC-{i:06d}.\nThis document contains proprietary information regarding {tenant} operations and governance policies."
        p2 = f"Detailed Analysis:\nIn fiscal evaluation period {2026 - (i % 5)}, operational key performance indicators demonstrated a {(i % 30) + 10}% improvement in process efficiency. Specific benchmarks require mandatory authorization prior to retrieval."
        p3 = f"Executive Governance Summary:\nAll associated personnel must adhere to organizational compliance mandates. Unapproved disclosure across organizational boundaries constitutes a violation of confidentiality."
        
        content = f"{p1}\n\n{p2}\n\n{p3}\n"
        with open(doc_path, 'w', encoding='utf-8') as f:
            f.write(content)
        
        total_generated += 1
        total_bytes += len(content)

print(f"Generated {total_generated} multi-tenant documents ({total_bytes / (1024*1024):.2f} MB across {len(TENANTS)} tenants) in {BENCHMARK_DIR}")
