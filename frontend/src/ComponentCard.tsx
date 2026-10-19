import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { type RegistryComponent, getStoryUrl } from "./data";

export function ComponentCard({ item }: { item: RegistryComponent }) {
  const [copied, setCopied] = useState(false);
  const target = getStoryUrl(item.chapter, item.display_name);

  const copyCommand = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    navigator.clipboard.writeText(`cargo ely add ${item.name}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="comp-card">
      <Link to="/components/$page/$story/" params={target} className="comp-card-preview" title={`Open ${item.display_name} demo`}>
        <VisualPreview category={item.category} name={item.name} displayName={item.display_name} />
        <div className="comp-card-hover-overlay">
          <span className="comp-hover-badge">Open Interactive Demo →</span>
        </div>
      </Link>

      <div className="comp-card-body">
        <div className="comp-card-header">
          <Link to="/components/$page/$story/" params={target} className="comp-card-title">
            {item.display_name}
          </Link>
          <span className="comp-card-chap">Ch {String(item.chapter).padStart(2, "0")}</span>
        </div>

        <p className="comp-card-desc">{item.description}</p>

        <div className="comp-card-footer">
          <code className="comp-card-type">{item.export_type}</code>
          <button type="button" className={`comp-copy-btn ${copied ? "copied" : ""}`} onClick={copyCommand} title={`Copy 'cargo ely add ${item.name}'`}>
            {copied ? "✓ Copied" : "cargo ely add"}
          </button>
        </div>
      </div>
    </div>
  );
}

function VisualPreview({ category, name }: { category: string; name: string; displayName: string }) {
  const n = name.toLowerCase();

  // 1. Buttons & Actions
  if (category === "buttons" || n.includes("button")) {
    return (
      <div className="preview-canvas preview-buttons">
        <span className="mini-btn-primary">Primary</span>
        <span className="mini-btn-outline">Outline</span>
        <span className="mini-btn-icon">⚡</span>
      </div>
    );
  }

  // 2. Forms & Inputs
  if (category === "forms" || n.includes("input") || n.includes("picker") || n.includes("select") || n.includes("slider") || n.includes("switch") || n.includes("check")) {
    if (n.includes("switch") || n.includes("toggle")) {
      return (
        <div className="preview-canvas">
          <div className="mini-switch on"><div className="mini-thumb" /></div>
          <div className="mini-switch off"><div className="mini-thumb" /></div>
        </div>
      );
    }
    if (n.includes("check")) {
      return (
        <div className="preview-canvas">
          <div className="mini-checkbox checked">✓</div>
          <div className="mini-line w-40" />
        </div>
      );
    }
    if (n.includes("slider")) {
      return (
        <div className="preview-canvas">
          <div className="mini-slider"><div className="mini-slider-fill" /><div className="mini-slider-thumb" /></div>
        </div>
      );
    }
    return (
      <div className="preview-canvas">
        <div className="mini-input">
          <span>Search...</span>
          <span className="mini-caret" />
        </div>
      </div>
    );
  }

  // 3. Layout
  if (category === "layout" || n.includes("layout") || n.includes("shell") || n.includes("split") || n.includes("dock") || n.includes("sidebar") || n.includes("card")) {
    return (
      <div className="preview-canvas">
        <div className="mini-wireframe-layout">
          <div className="mini-wf-header" />
          <div className="mini-wf-body">
            <div className="mini-wf-side" />
            <div className="mini-wf-content">
              <div className="mini-line w-60" />
              <div className="mini-line w-40" />
            </div>
            {n.includes("app_layout") && <div className="mini-wf-side right" />}
          </div>
          <div className="mini-wf-footer" />
        </div>
      </div>
    );
  }

  // 4. Window & Shell
  if (category === "shell" || n.includes("window") || n.includes("titlebar")) {
    return (
      <div className="preview-canvas">
        <div className="mini-window-frame">
          <div className="mini-dots"><span className="dot r" /><span className="dot y" /><span className="dot g" /></div>
          <div className="mini-line w-30" />
        </div>
      </div>
    );
  }

  // 5. Overlays
  if (category === "overlays" || n.includes("dialog") || n.includes("palette") || n.includes("modal") || n.includes("popover") || n.includes("sheet")) {
    if (n.includes("palette")) {
      return (
        <div className="preview-canvas">
          <div className="mini-cmd-palette">
            <span className="mini-search-icon">🔍</span>
            <span className="mini-cmd-text">Quick search...</span>
            <kbd className="mini-kbd">⌘K</kbd>
          </div>
        </div>
      );
    }
    return (
      <div className="preview-canvas">
        <div className="mini-modal-card">
          <div className="mini-modal-header" />
          <div className="mini-line w-50" />
          <div className="mini-btn-primary sm">Confirm</div>
        </div>
      </div>
    );
  }

  // 6. Charts & Finance
  if (category === "charts" || category === "finance" || n.includes("chart")) {
    if (n.includes("candlestick") || n.includes("ohlc") || n.includes("trade") || n.includes("depth")) {
      return (
        <div className="preview-canvas">
          <svg className="mini-chart-svg" viewBox="0 0 100 40">
            <line x1="20" y1="5" x2="20" y2="35" stroke="var(--muted)" strokeWidth="1" />
            <rect x="16" y="10" width="8" height="15" fill="#10b981" rx="1" />
            <line x1="45" y1="8" x2="45" y2="38" stroke="var(--muted)" strokeWidth="1" />
            <rect x="41" y="12" width="8" height="18" fill="#ef4444" rx="1" />
            <line x1="70" y1="4" x2="70" y2="32" stroke="var(--muted)" strokeWidth="1" />
            <rect x="66" y="8" width="8" height="14" fill="#10b981" rx="1" />
          </svg>
        </div>
      );
    }
    return (
      <div className="preview-canvas">
        <svg className="mini-chart-svg" viewBox="0 0 100 40">
          <path d="M5,35 Q30,10 50,25 T95,8" fill="none" stroke="var(--accent)" strokeWidth="2.5" strokeLinecap="round" />
          <circle cx="95" cy="8" r="3" fill="var(--accent)" />
        </svg>
      </div>
    );
  }

  // 7. Terminal & Editor
  if (category === "terminal" || category === "editor" || n.includes("code") || n.includes("terminal")) {
    return (
      <div className="preview-canvas">
        <div className="mini-terminal-box">
          <span className="mini-term-prompt">$</span>
          <span className="mini-term-code">ely-gpui init</span>
          <span className="mini-term-cursor" />
        </div>
      </div>
    );
  }

  // 8. Tables
  if (category === "tables" || n.includes("table") || n.includes("grid")) {
    return (
      <div className="preview-canvas">
        <div className="mini-table-grid">
          <div className="mini-table-head" />
          <div className="mini-table-row" />
          <div className="mini-table-row" />
        </div>
      </div>
    );
  }

  // 9. AI Chat & Agent
  if (category === "chat" || category === "agent" || n.includes("chat") || n.includes("agent") || n.includes("message")) {
    return (
      <div className="preview-canvas">
        <div className="mini-chat-stream">
          <div className="mini-bubble user" />
          <div className="mini-bubble ai">✨</div>
        </div>
      </div>
    );
  }

  // 10. Data Display & Badges
  if (category === "data" || category === "data_display" || n.includes("badge") || n.includes("avatar")) {
    return (
      <div className="preview-canvas">
        <div className="mini-avatar-wrap">
          <div className="mini-avatar">EV</div>
          <span className="mini-badge-pill">Active</span>
        </div>
      </div>
    );
  }

  // 11. Default Geometric Specimen
  return (
    <div className="preview-canvas">
      <div className="mini-geometric-glyph">
        <span className="glyph-symbol">◇</span>
        <div className="mini-line w-40" />
      </div>
    </div>
  );
}
