"use client";

import { useMemo, useState } from "react";
import {
  buildDistributionTree,
  getDistributionProof,
  verifyDistributionProof,
  type DistributionReceipt,
} from "@indian-rwa/registry-sdk/distribution";

const receipts: DistributionReceipt[] = [
  { investorRef: bytes(31), gross: 125000n, withheld: 12500n, net: 112500n, nonce: 1n },
  { investorRef: bytes(32), gross: 98000n, withheld: 9800n, net: 88200n, nonce: 2n },
  { investorRef: bytes(33), gross: 142000n, withheld: 14200n, net: 127800n, nonce: 3n },
].map((receipt) => ({
  ...receipt, programId: bytes(7), asset: bytes(11), epochId: 202611n, currencyCode: "INR", decimals: 2,
}));

const tabs = ["Issuer", "Registrar", "Investor", "Auditor"] as const;
type Tab = typeof tabs[number];

export default function Home() {
  const [tab, setTab] = useState<Tab>("Issuer");
  const [tampered, setTampered] = useState(false);
  const tree = useMemo(() => buildDistributionTree(receipts), []);
  const receipt = tampered ? { ...receipts[1], net: receipts[1].net - 1n } : receipts[1];
  const proofValid = verifyDistributionProof(receipt, getDistributionProof(tree, 1), tree.root);

  return (
    <main className="shell">
      <header className="topbar">
        <a className="brand" href="#top" aria-label="RWA Registry home"><span className="brand-mark">R</span><span>Registry<span className="brand-light"> / Protocol</span></span></a>
        <div className="network"><span className="pulse" /> LOCAL SCENARIO <span className="network-divider">·</span> NO TX SUBMITTED</div>
        <button className="wallet-button" disabled title="Wallet connection is not part of this demo slice">Connect wallet <span>↗</span></button>
      </header>

      <section className="hero" id="top">
        <div>
          <div className="eyebrow"><span className="eyebrow-line" /> INDIAN REAL ESTATE · PROTOTYPE</div>
          <h1>Evidence that<br /><em>travels together.</em></h1>
          <p className="hero-copy">A shared record for property documents, offering events, and rental distribution receipts.</p>
        </div>
        <div className="hero-side">
          <span className="side-label">DEMO ASSET</span>
          <span className="asset-code">BKC-DEMO-001</span>
          <span className="asset-place">Fictional commercial property<br />Mumbai, Maharashtra</span>
          <span className="status-chip"><span /> SIMULATED DATA</span>
        </div>
      </section>

      <nav className="tabs" aria-label="Demo roles">
        {tabs.map((item) => <button key={item} className={`tab ${tab === item ? "active" : ""}`} onClick={() => setTab(item)}>{item}</button>)}
        <span className="tab-note">ROLE VIEWS · ALL DATA FICTIONAL</span>
      </nav>

      {tab === "Issuer" && <IssuerView />}
      {tab === "Registrar" && <RegistrarView />}
      {tab === "Investor" && <InvestorView
        tampered={tampered} toggleTamper={() => setTampered((value) => !value)} proofValid={proofValid}
        root={hex(tree.root)} receipt={receipt}
      />}
      {tab === "Auditor" && <AuditorView root={hex(tree.root)} />}

      <footer><span>OPEN PROTOCOL · FICTIONAL DATA</span><span>HASHES CHECK BYTES. THEY DO NOT PROVE TITLE OR PAYMENT.</span></footer>
    </main>
  );
}

function IssuerView() {
  return <section className="content-grid">
    <div className="panel wide">
      <PanelHeading kicker="PROPERTY DOCUMENT REGISTRY" title="Evidence history" action="2 RECORDS" />
      <div className="timeline">
        <div className="timeline-row"><span className="timeline-dot verified" /><div className="timeline-main"><strong>Valuation report · v2</strong><span>Issuer commitment · added 18 Nov 2026</span></div><code>sha256 · 9d2a…71c8</code><span className="tag attested">ATTESTED</span></div>
        <div className="timeline-row"><span className="timeline-dot" /><div className="timeline-main"><strong>Valuation report · v1</strong><span>Superseded by v2 · added 04 Nov 2026</span></div><code>sha256 · 20bf…a10e</code><span className="tag">SUPERSEDED</span></div>
        <div className="timeline-row"><span className="timeline-dot" /><div className="timeline-main"><strong>Title-search report · v1</strong><span>Issuer commitment · added 01 Nov 2026</span></div><code>sha256 · c622…03fa</code><span className="tag">COMMITMENT</span></div>
      </div>
      <div className="panel-foot">Document hashes verify bytes only. They do not prove authenticity, title, or legal status.</div>
    </div>
    <div className="panel summary-panel"><PanelHeading kicker="ISSUER VIEW" title="Asset record" /><dl className="detail-list"><div><dt>Asset ID</dt><dd>BKC-DEMO-001</dd></div><div><dt>Issuer</dt><dd>Harborline Demo SPV</dd></div><div><dt>Linked mint</dt><dd className="muted">None in prototype</dd></div><div><dt>Registry status</dt><dd><span className="soft-dot" /> Records shown locally</dd></div></dl><div className="notice">The demo UI is a read-only scenario preview. It has not submitted a transaction.</div></div>
  </section>;
}

function RegistrarView() {
  return <section className="content-grid">
    <div className="panel wide"><PanelHeading kicker="PRIVATE PLACEMENT REGISTRY" title="Offering funnel" action="FY 2026–27 · LABEL ONLY" /><div className="funnel">
      {["Registered references", "Invitations recorded", "Applications recorded", "Allotments recorded"].map((label, i) => <div className="funnel-row" key={label}><span>{label}</span><div className="funnel-track"><i style={{ width: `${[100, 75, 50, 37.5][i]}%` }} /></div><b>{[8, 6, 4, 3][i].toString().padStart(2, "0")}</b></div>)}
    </div><div className="panel-foot">One synthetic person can map to multiple wallets off-chain and still counts once. These counts represent references, not wallet holders.</div></div>
    <div className="panel summary-panel"><PanelHeading kicker="OFFERING" title="Scenario details" /><dl className="detail-list"><div><dt>Offering ref</dt><dd>OFFER-DEMO-01</dd></div><div><dt>Terms commitment</dt><dd>sha256 · 23a1…c091</dd></div><div><dt>Issuer limit</dt><dd>12 references</dd></div><div><dt>Legal interpretation</dt><dd className="muted">Not encoded</dd></div></dl><div className="notice">The configured limit is an issuer-defined administrative guard, not a statutory cap.</div></div>
  </section>;
}

function InvestorView({ tampered, toggleTamper, proofValid, root, receipt }: { tampered: boolean; toggleTamper: () => void; proofValid: boolean; root: string; receipt: DistributionReceipt }) {
  return <section className="content-grid">
    <div className="panel wide"><PanelHeading kicker="RENTAL YIELD & TAX RECEIPT REGISTRY" title="Receipt proof" action="EPOCH 2026-11" /><div className="receipt-card"><div className="receipt-top"><div><span className="side-label">FICTIONAL RECEIPT</span><strong>INVESTOR-REF-20</strong></div><span className={`proof-pill ${proofValid ? "ok" : "fail"}`}><span /> {proofValid ? "INCLUSION PROOF VALID" : "PROOF INVALID"}</span></div><div className="receipt-amount"><span>Net entitlement</span><strong>₹{formatAmount(receipt.net)}</strong><small>INR · synthetic smallest units · 2 decimals</small></div><div className="receipt-split"><div><span>Gross</span><b>₹{formatAmount(receipt.gross)}</b></div><div><span>Withheld assertion</span><b>₹{formatAmount(receipt.withheld)}</b></div><div><span>Net</span><b>₹{formatAmount(receipt.net)}</b></div></div><div className="root-block"><span>COMMITTED MERKLE ROOT</span><code>{root}</code></div></div><button className="secondary-button" onClick={toggleTamper}>{tampered ? "Restore original receipt" : "Change amount to test proof"}<span>↗</span></button></div>
    <div className="panel summary-panel"><PanelHeading kicker="VERIFY LOCALLY" title="What this proves" /><div className="verification-state"><span className={`check-mark ${proofValid ? "valid" : "invalid"}`}>{proofValid ? "✓" : "×"}</span><div><strong>{proofValid ? "Receipt included" : "Receipt was changed"}</strong><span>{proofValid ? "This receipt matches the local demo root." : "The modified amount no longer matches the root."}</span></div></div><div className="notice">Inclusion is not proof of rent received, bank transfer settled, or tax remitted. The withholding figure is issuer-supplied.</div></div>
  </section>;
}

function AuditorView({ root }: { root: string }) {
  return <section className="content-grid">
    <div className="panel wide"><PanelHeading kicker="AUDIT VIEW" title="Committed evidence" action="SCENARIO PREVIEW" /><div className="audit-list"><div><span>Property record</span><code>asset · BKC-DEMO-001</code><b>3 document versions</b></div><div><span>Offering record</span><code>offering · OFFER-DEMO-01</code><b>8 unique references</b></div><div><span>Distribution epoch</span><code>epoch · 2026-11</code><b>3 receipt leaves</b></div><div><span>Merkle root</span><code>{root}</code><b>Locally reproducible</b></div></div></div>
    <div className="panel summary-panel"><PanelHeading kicker="TRUST BOUNDARY" title="What remains off-chain" /><ul className="boundary-list"><li>Document authenticity and property title</li><li>Investor identity and eligibility</li><li>Legal offering classification</li><li>Rental accounting and tax treatment</li><li>Bank transfers and remittance</li></ul><div className="notice">No green compliance, title-verified, or tax-paid status is inferred from these records.</div></div>
  </section>;
}

function PanelHeading({ kicker, title, action }: { kicker: string; title: string; action?: string }) {
  return <div className="panel-heading"><div><span className="panel-kicker">{kicker}</span><h2>{title}</h2></div>{action && <span className="panel-action">{action}</span>}</div>;
}

function bytes(value: number): Uint8Array { return new Uint8Array(32).fill(value); }
function hex(value: Uint8Array): string { return Array.from(value).map((byte) => byte.toString(16).padStart(2, "0")).join(""); }
function formatAmount(value: bigint): string { return (Number(value) / 100).toLocaleString("en-IN", { minimumFractionDigits: 2, maximumFractionDigits: 2 }); }
