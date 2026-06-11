import React, { useEffect } from "react";
import { Download, Github } from "lucide-react";
import { GhostLogo } from "./GhostLogo";
import "./LandingPage.css";

const downloadUrl = "/downloads/GhostZip_0.1.0_x64-setup.exe";

export function LandingPage() {
  useEffect(() => {
    // Basic interaction for how it works panels
    const steps = document.querySelectorAll(".hstep");
    const panels = document.querySelectorAll(".hv-panel");
    steps.forEach((step, idx) => {
      step.addEventListener("click", () => {
        steps.forEach((s) => s.classList.remove("hs-active"));
        panels.forEach((p) => p.classList.remove("hv-active"));
        step.classList.add("hs-active");
        if (panels[idx]) panels[idx].classList.add("hv-active");
      });
    });
  }, []);

  return (
    <div className="ghostzip-landing">
      {/* ── NAV ── */}
      <nav>
        <div className="nl">
          <div className="nlogo">
            <GhostLogo size={30} />
            GhostZip
          </div>
          <div className="nbadge">BETA</div>
          <div className="nlinks" style={{ marginLeft: 30 }}>
            <a href="#how">How it Works</a>
            <a href="#compare">Compare</a>
            <a href="#features">Features</a>
          </div>
        </div>
        <div className="nr">
          <a href="https://github.com/Rudra1959/GhostZip" target="_blank" rel="noreferrer" className="btn-gh">
            <Github size={15} />
            Source
          </a>
          <a href={downloadUrl} className="btn-nav-dl" download>Download</a>
        </div>
      </nav>

      {/* ── HERO ── */}
      <section className="hero">
        <div className="hgrid"></div>
        <div className="hglow"></div>
        <div className="hglow2"></div>
        <div className="eyebrow">
          <span className="pulse"></span>
          Extract on a completely full drive
        </div>
        <h1>
          Extract huge files <br />
          <span className="dim">with tiny free space.</span>
        </h1>
        <p className="hsub">
          GhostZip extracts archives directly into themselves. No temp folder. No extra space.
          It writes one file, then purges the source data to make room for the next.
        </p>
        <div className="hbtns">
          <a href={downloadUrl} className="btn-dl" download>
            <Download size={19} />
            Download for Windows
          </a>
          <a href="https://github.com/Rudra1959/GhostZip" target="_blank" rel="noreferrer" className="btn-gh2">
            <Github size={18} />
            View Source
          </a>
        </div>
        <div className="hmeta">
          <span>✓ Preflight disk check</span>
          <span>✓ ZIP Extract & Purge</span>
          <span>✓ Open Source</span>
        </div>

        {/* ── BIG VISUAL DEMO ── */}
        <div className="demo-outer">
          <div className="demo-card">
            <div className="demo-bar">
              <div className="ddots">
                <div className="ddot" style={{ background: '#ff5f56' }}></div>
                <div className="ddot" style={{ background: '#ffbd2e' }}></div>
                <div className="ddot" style={{ background: '#27c93f' }}></div>
              </div>
              <div className="demo-title-bar">
                <span>GhostZip</span>
                <span style={{ opacity: 0.5 }}>— Live Run</span>
              </div>
            </div>
            <div className="demo-body">
              <div className="demo-left">
                <div className="demo-sec-label">Drive C: Storage</div>
                <div className="stor-row">
                  <span>Capacity: 500 GB</span>
                  <span className="stor-free">1.2 GB Free</span>
                </div>
                <div className="stor-track">
                  <div className="stor-fill" style={{ width: '98%', background: 'var(--red)' }}></div>
                </div>

                <div className="demo-sec-label" style={{ marginTop: 24 }}>Archive Contents</div>
                <div className="flist">
                  <div className="fitem fdone">
                    <div className="ficon">📄</div>
                    <div className="fname">assets_bundle.pack</div>
                    <div className="fsize">14.2 GB</div>
                    <div className="fbadge fb-done">Done</div>
                  </div>
                  <div className="fitem factive">
                    <div className="ficon">🎞️</div>
                    <div className="fname">renders.mp4</div>
                    <div className="fsize">10.8 GB</div>
                    <div className="fbadge fb-act">Purging</div>
                  </div>
                  <div className="fitem fwait">
                    <div className="ficon">🗄️</div>
                    <div className="fname">db_backup.sql</div>
                    <div className="fsize">7.4 GB</div>
                    <div className="fbadge fb-wait">Queued</div>
                  </div>
                </div>
              </div>
              <div className="demo-right">
                <div className="demo-sec-label">GhostZip Output</div>
                <div className="arch-visual">
                  <div className="arch-particles">
                    <div className="arch-particle" style={{ left: '20%', animationDelay: '0s', background: 'rgba(0,229,192,0.1)', color: 'var(--teal)' }}>0110</div>
                    <div className="arch-particle" style={{ left: '50%', animationDelay: '1s', background: 'rgba(0,229,192,0.1)', color: 'var(--teal)' }}>1001</div>
                    <div className="arch-particle" style={{ left: '80%', animationDelay: '2s', background: 'rgba(0,229,192,0.1)', color: 'var(--teal)' }}>1110</div>
                  </div>
                  <div className="arch-bar-wrap">
                    <div className="arch-bar-track">
                      <div className="arch-bar-fill" style={{ width: '60%' }}></div>
                    </div>
                    <div className="arch-bar-label">
                      <span>Source.zip</span>
                      <span>Shrinking...</span>
                    </div>
                  </div>
                </div>
                <div className="stats-row">
                  <div className="stat-box">
                    <div className="stat-label">Total Unpacked</div>
                    <div className="stat-val">32.4 GB</div>
                  </div>
                  <div className="stat-box">
                    <div className="stat-label">Space Saved</div>
                    <div className="stat-val green">14.2 GB</div>
                  </div>
                </div>
              </div>
            </div>
            <div className="demo-footer">
              <div className="gprog-track">
                <div className="gprog-fill" style={{ width: '45%' }}></div>
              </div>
              <div className="gprog-info">45% Complete</div>
            </div>
          </div>
        </div>
      </section>

      {/* ── THE CORE CONCEPT ── */}
      <section className="sec sec-dark" id="concept">
        <div className="ctr">
          <div className="ew">The Concept</div>
          <h2 className="sh">Traditional extraction requires double the space.</h2>
          <p className="sp">Normal extractors pull data out of the archive and write it to disk. This means for a brief time, both the archive and the extracted files exist on the drive simultaneously.</p>
          
          <div className="concept-grid">
            <div>
              <div className="concept-side-label">Traditional Extraction</div>
              <div className="concept-block bad">
                <div className="cb-header">
                  <div className="cb-tag bad">Fails</div>
                  <div style={{ fontSize: 13, color: 'var(--muted)', fontWeight: 600 }}>Standard Extractor</div>
                </div>
                <div className="cb-body">
                  <div className="drive-legend">
                    <div><span className="dl-dot" style={{ background: '#5b8fff' }}></span>Archive</div>
                    <div><span className="dl-dot" style={{ background: '#00e5c0' }}></span>Output</div>
                    <div><span className="dl-dot" style={{ background: '#ff5252' }}></span>Over Limit</div>
                  </div>
                  <div className="drive-viz">
                    <div className="drive-seg" style={{ width: '50%', background: 'rgba(91,143,255,0.4)', color: 'var(--white)' }}>Archive (50GB)</div>
                    <div className="drive-seg" style={{ width: '50%', background: 'rgba(255,82,82,0.4)', color: 'var(--white)' }}>Output (50GB)</div>
                  </div>
                  <div className="drive-note bad-note">
                    Requires <b>100GB</b> total space. Disk full error.
                  </div>
                </div>
              </div>
            </div>

            <div>
              <div className="concept-side-label">GhostZip Extraction</div>
              <div className="concept-block good">
                <div className="cb-header">
                  <div className="cb-tag good">Succeeds</div>
                  <div style={{ fontSize: 13, color: 'var(--muted)', fontWeight: 600 }}>GhostZip Purge</div>
                </div>
                <div className="cb-body">
                  <div className="drive-legend">
                    <div><span className="dl-dot" style={{ background: '#5b8fff' }}></span>Archive</div>
                    <div><span className="dl-dot" style={{ background: '#00e5c0' }}></span>Output</div>
                  </div>
                  <div className="drive-viz">
                    <div className="drive-seg" style={{ width: '25%', background: 'rgba(91,143,255,0.4)', color: 'var(--white)' }}>Shrinking ZIP</div>
                    <div className="drive-seg" style={{ width: '25%', background: 'rgba(0,229,192,0.4)', color: '#000' }}>Extracted Files</div>
                    <div className="drive-seg" style={{ width: '50%', background: 'var(--surf)', color: 'var(--muted)' }}>Free Space Maintained</div>
                  </div>
                  <div className="drive-note good-note">
                    Requires only <b>~51GB</b> total space. Smooth extraction.
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ── PROBLEM / SOLUTION COMPARE ── */}
      <section className="sec sec-darker" id="compare">
        <div className="ctr">
          <div className="split-compare">
            <div className="sc-col bad-col">
              <div className="sc-tag bad">Without GhostZip</div>
              <h3 className="sc-h">Archive + output + temp files</h3>
              <p className="sc-p">A 60 GB archive can require well over 120 GB of working space before cleanup happens.</p>
              <div className="sc-items">
                <div className="sc-item">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--red)" strokeWidth="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
                  Fails late in the process
                </div>
                <div className="sc-item">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--red)" strokeWidth="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
                  Temp-folder pressure on C: drive
                </div>
                <div className="sc-item">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--red)" strokeWidth="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
                  Manual cleanup required
                </div>
              </div>
            </div>
            <div className="sc-divider">
              <div className="vs-pill">VS</div>
            </div>
            <div className="sc-col good-col">
              <div className="sc-tag good">With GhostZip</div>
              <h3 className="sc-h">Analyze, extract, purge</h3>
              <p className="sc-p">GhostZip reads the manifest, writes one verified file, then makes room for the next one.</p>
              <div className="sc-items">
                <div className="sc-item">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--teal)" strokeWidth="2"><polyline points="20 6 9 17 4 12"/></svg>
                  Fails early if unsafe
                </div>
                <div className="sc-item">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--teal)" strokeWidth="2"><polyline points="20 6 9 17 4 12"/></svg>
                  Uses only the selected drive
                </div>
                <div className="sc-item">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--teal)" strokeWidth="2"><polyline points="20 6 9 17 4 12"/></svg>
                  Zero extra space required
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ── HOW IT WORKS ── */}
      <section className="sec sec-dark" id="how">
        <div className="ctr">
          <div className="ew">The Engine</div>
          <h2 className="sh">Four steps. No magic. Just math.</h2>
          <p className="sp">GhostZip uses the filesystem's sparse file capabilities to release space back to the OS immediately after extracting a file from the archive.</p>

          <div className="how-grid">
            <div className="how-steps-list">
              <div className="hstep hs-active">
                <div className="hstep-left">
                  <div className="hstep-num">1</div>
                  <div className="hstep-conn"></div>
                </div>
                <div className="hstep-body">
                  <div className="hstep-h">Pre-flight Analysis</div>
                  <div className="hstep-p">We analyze the entire archive before touching the disk. We calculate exactly how much space is needed and verify it against your current disk space.</div>
                  <div className="hstep-chip">Safety First</div>
                </div>
              </div>
              <div className="hstep">
                <div className="hstep-left">
                  <div className="hstep-num">2</div>
                  <div className="hstep-conn"></div>
                </div>
                <div className="hstep-body">
                  <div className="hstep-h">Stream Extraction</div>
                  <div className="hstep-p">We extract a single file directly to its final destination. No temporary folders are used.</div>
                  <div className="hstep-chip">Zero Temp Files</div>
                </div>
              </div>
              <div className="hstep">
                <div className="hstep-left">
                  <div className="hstep-num">3</div>
                  <div className="hstep-conn"></div>
                </div>
                <div className="hstep-body">
                  <div className="hstep-h">Verify & Punch Hole</div>
                  <div className="hstep-p">Once verified, we ask the OS to "punch a hole" in the original archive file where that extracted data used to be, instantly freeing up disk space.</div>
                  <div className="hstep-chip">Sparse Files</div>
                </div>
              </div>
              <div className="hstep">
                <div className="hstep-left">
                  <div className="hstep-num">4</div>
                  <div className="hstep-conn"></div>
                </div>
                <div className="hstep-body">
                  <div className="hstep-h">Finalize</div>
                  <div className="hstep-p">The archive size shrinks as files are extracted. When done, you have all your files and the empty shell of the archive is removed.</div>
                  <div className="hstep-chip">Done</div>
                </div>
              </div>
            </div>

            <div className="how-visual">
              <div className="hv-top">Visualizer</div>
              <div className="hv-panels">
                <div className="hv-panel hv-active" id="hvp1">
                  <p style={{ fontSize: 13, color: 'var(--muted)', marginBottom: 16 }}>Sequential format rewrite — files stored in strict linear order.</p>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                    <div style={{ height: 20, background: 'linear-gradient(90deg,#1a2a1a,#0a3a2a)', borderRadius: 4, display: 'flex', alignItems: 'center', padding: '0 8px', fontSize: 10, color: 'var(--teal)' }}>▓▓▓▓▓▓▓▓▓▓ File 1 — assets_bundle.pack (14.2 GB)</div>
                    <div style={{ height: 20, background: 'linear-gradient(90deg,#1a2a1a,#0a3a2a)', borderRadius: 4, display: 'flex', alignItems: 'center', padding: '0 8px', fontSize: 10, color: 'var(--teal)', opacity: 0.8 }}>▓▓▓▓▓▓▓ File 2 — renders.mp4 (10.8 GB)</div>
                    <div style={{ height: 20, background: 'linear-gradient(90deg,#1a2a1a,#0a3a2a)', borderRadius: 4, display: 'flex', alignItems: 'center', padding: '0 8px', fontSize: 10, color: 'var(--teal)', opacity: 0.6 }}>▓▓▓▓▓ File 3 — db_backup.sql (7.4 GB)</div>
                    <div style={{ height: 20, background: 'linear-gradient(90deg,#1a2a1a,#0a3a2a)', borderRadius: 4, display: 'flex', alignItems: 'center', padding: '0 8px', fontSize: 10, color: 'var(--teal)', opacity: 0.4 }}>▓▓▓ File 4… (4.3 GB)</div>
                    <div style={{ height: 20, background: 'var(--surf2)', borderRadius: 4, display: 'flex', alignItems: 'center', padding: '0 8px', fontSize: 10, color: 'var(--muted)', opacity: 0.3 }}>▓▓ …and 43 more files</div>
                  </div>
                  <p style={{ fontSize: 11, color: 'var(--muted)', marginTop: 10 }}>Each file is a self-contained sequential block. This makes safe deletion possible.</p>
                </div>
                <div className="hv-panel" id="hvp2">
                  <p style={{ fontSize: 13, color: 'var(--muted)' }}>Extracting to destination...</p>
                </div>
                <div className="hv-panel" id="hvp3">
                  <p style={{ fontSize: 13, color: 'var(--muted)' }}>Punching hole in archive...</p>
                </div>
                <div className="hv-panel" id="hvp4">
                  <p style={{ fontSize: 13, color: 'var(--muted)' }}>Extraction complete!</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ── FEATURES ── */}
      <section className="sec sec-darker" id="features">
        <div className="ctr">
          <div className="ew">Highlights</div>
          <h2 className="sh">Serious archive handling.</h2>
          
          <div className="feat-grid">
            <div className="feat-cell">
              <div className="fc-icon">🛡️</div>
              <div className="fc-h">Unsafe paths blocked</div>
              <div className="fc-p">Traversal paths, absolute paths, and suspicious archive entries are rejected before they escape.</div>
            </div>
            <div className="feat-cell">
              <div className="fc-icon">💻</div>
              <div className="fc-h">Native Windows App</div>
              <div className="fc-p">Built with Tauri and Rust for extreme performance. Shell integration for right-click flows.</div>
            </div>
            <div className="feat-cell">
              <div className="fc-icon">📂</div>
              <div className="fc-h">Wide Format Support</div>
              <div className="fc-p">Supports ZIP, 7Z, RAR, TAR, TAR.GZ, and more. Purge mode currently optimized for ZIP.</div>
            </div>
          </div>
        </div>
      </section>

      {/* ── DOWNLOAD CTA ── */}
      <section className="cta-sec" id="download">
        <div className="cta-glow"></div>
        <div className="cta-glow2"></div>
        <h2>Ready to <em>extract?</em></h2>
        <p className="lead">Install GhostZip like a normal desktop app, then use it when a huge archive lands on a drive that is already almost full.</p>
        
        <div className="dl-box">
          <div className="dl-os-row">
            <div className="dl-os-icon">🪟</div>
            <div className="dl-os-info">
              <div className="dl-os-name">Windows 10 / 11</div>
              <div className="dl-os-sub">64-bit Installer</div>
            </div>
          </div>
          <a href={downloadUrl} className="btn-big-dl" download>
            <Download size={22} />
            Download GhostZip
          </a>
          <div className="dl-meta">Version 0.1.0 • 12 MB</div>
          
          <div className="trust-row">
            <span>✓ No ads</span>
            <span>✓ Open source</span>
            <span>✓ Tiny footprint</span>
          </div>
        </div>
      </section>

      {/* ── FOOTER ── */}
      <footer>
        <div className="fl">
          <div className="fc-logo">
            <GhostLogo size={18} />
            GhostZip
          </div>
          <div className="fl-sub">Extract archives with zero extra space.</div>
        </div>
        <div className="fm">
          <div className="fm-copy">© {new Date().getFullYear()} GhostZip Contributors. Released under MIT License.</div>
        </div>
        <div className="fr">
          <a href="https://github.com/Rudra1959/GhostZip">GitHub</a>
          <a href="https://github.com/Rudra1959/GhostZip/issues">Report Issue</a>
        </div>
      </footer>
    </div>
  );
}
