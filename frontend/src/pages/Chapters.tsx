import { useEffect, useMemo, useRef, useState } from "react";
import { ComponentCard } from "../ComponentCard";
import { Find } from "../Find";
import {
  type RegistryChapter,
  type RegistryComponent,
  pad,
  registryChapters,
  registryComponents,
} from "../data";

export function Chapters() {
  const [activeChapter, setActiveChapter] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");

  useEffect(() => {
    document.title = activeChapter
      ? `${registryChapters.find((c) => c.number === activeChapter)?.title} · Components · Ely`
      : "Components (1,280) · Ely";
  }, [activeChapter]);

  // Filter components by search query or chapter
  const filteredComponents = useMemo(() => {
    let list = registryComponents;
    if (activeChapter !== null) {
      list = list.filter((c) => c.chapter === activeChapter);
    }
    if (query.trim()) {
      const q = query.trim().toLowerCase().replace("-", "_");
      list = list.filter(
        (c) =>
          c.name.toLowerCase().includes(q) ||
          c.display_name.toLowerCase().includes(q) ||
          c.description.toLowerCase().includes(q) ||
          c.category.toLowerCase().includes(q)
      );
    }
    return list;
  }, [activeChapter, query]);

  // Group by chapter
  const displayedChapters = useMemo(() => {
    if (activeChapter !== null) {
      const ch = registryChapters.find((c) => c.number === activeChapter);
      return ch ? [ch] : [];
    }
    if (query.trim()) {
      // Only include chapters that have matching components
      const activeNums = new Set(filteredComponents.map((c) => c.chapter));
      return registryChapters.filter((c) => activeNums.has(c.number));
    }
    return registryChapters;
  }, [activeChapter, query, filteredComponents]);

  const activeChapterData = activeChapter
    ? registryChapters.find((c) => c.number === activeChapter)
    : null;

  return (
    <div className="components-layout">
      {/* Sidebar with Chapter selector */}
      <aside className="components-sidebar" data-lenis-prevent>
        <div className="sidebar-search-wrap">
          <Find
            className="find small"
            query={query}
            onQuery={setQuery}
            onEnter={() => {}}
          />
        </div>

        <div className="sidebar-chapter-list">
          <button
            type="button"
            className={`chapter-nav-item all-chapters ${activeChapter === null ? "active" : ""}`}
            onClick={() => setActiveChapter(null)}
          >
            <span className="chapter-nav-title">All Chapters</span>
            <span className="chapter-nav-badge">{registryComponents.length}</span>
          </button>

          <div className="sidebar-divider" />

          {registryChapters.map((ch) => {
            const isActive = activeChapter === ch.number;
            return (
              <button
                key={ch.number}
                type="button"
                className={`chapter-nav-item ${isActive ? "active" : ""}`}
                onClick={() => setActiveChapter(ch.number)}
              >
                <span className="chapter-nav-num">{pad(ch.number)}</span>
                <span className="chapter-nav-title">{ch.title}</span>
                <span className="chapter-nav-badge">{ch.component_count}</span>
              </button>
            );
          })}
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="components-content">
        <header className="components-header">
          <div className="components-header-info">
            <span className="components-header-overline">
              {activeChapterData
                ? `Chapter ${pad(activeChapterData.number)}`
                : "All Chapters"}
            </span>
            <h1 className="components-header-title">
              {activeChapterData ? activeChapterData.title : "Components"}
            </h1>
            <p className="components-header-desc">
              {activeChapterData
                ? activeChapterData.summary
                : `Interactive picture view of all ${registryComponents.length} components across ${registryChapters.length} chapters.`}
            </p>
          </div>

          <div className="components-header-actions">
            <div className="view-toggle">
              <button
                type="button"
                className={`view-btn ${viewMode === "grid" ? "active" : ""}`}
                onClick={() => setViewMode("grid")}
                title="Picture View (Grid)"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="3" y="3" width="7" height="7" rx="1" />
                  <rect x="14" y="3" width="7" height="7" rx="1" />
                  <rect x="3" y="14" width="7" height="7" rx="1" />
                  <rect x="14" y="14" width="7" height="7" rx="1" />
                </svg>
                Picture View
              </button>
              <button
                type="button"
                className={`view-btn ${viewMode === "list" ? "active" : ""}`}
                onClick={() => setViewMode("list")}
                title="Dense List View"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="8" y1="6" x2="21" y2="6" />
                  <line x1="8" y1="12" x2="21" y2="12" />
                  <line x1="8" y1="18" x2="21" y2="18" />
                  <line x1="3" y1="6" x2="3.01" y2="6" />
                  <line x1="3" y1="12" x2="3.01" y2="12" />
                  <line x1="3" y1="18" x2="3.01" y2="18" />
                </svg>
                List
              </button>
            </div>
          </div>
        </header>

        {query && (
          <div className="search-stats-banner">
            Found <strong>{filteredComponents.length}</strong> components matching &ldquo;{query}&rdquo;
          </div>
        )}

        {filteredComponents.length === 0 ? (
          <div className="empty-results-box">
            <p>No components match &ldquo;{query}&rdquo;.</p>
            <button
              type="button"
              className="text"
              onClick={() => {
                setQuery("");
                setActiveChapter(null);
              }}
            >
              Reset filters
            </button>
          </div>
        ) : (
          <div className="chapters-sections-wrap">
            {displayedChapters.map((ch) => {
              const chComps = filteredComponents.filter(
                (c) => c.chapter === ch.number
              );
              if (chComps.length === 0) return null;

              return (
                <LazyChapterSection
                  key={ch.number}
                  chapter={ch}
                  components={chComps}
                  viewMode={viewMode}
                  standalone={activeChapter !== null}
                />
              );
            })}
          </div>
        )}
      </main>
    </div>
  );
}

/** Lazy chapter section that only mounts its component cards when visible in the viewport */
function LazyChapterSection({
  chapter,
  components,
  viewMode,
  standalone,
}: {
  chapter: RegistryChapter;
  components: RegistryComponent[];
  viewMode: "grid" | "list";
  standalone: boolean;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(standalone);

  useEffect(() => {
    if (standalone || isVisible) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "600px 0px" }
    );

    if (containerRef.current) {
      observer.observe(containerRef.current);
    }

    return () => observer.disconnect();
  }, [standalone, isVisible]);

  return (
    <section ref={containerRef} className="chapter-display-section">
      {!standalone && (
        <div className="chapter-section-header">
          <div className="chapter-section-badge">{pad(chapter.number)}</div>
          <div>
            <h2 className="chapter-section-title">{chapter.title}</h2>
            <p className="chapter-section-summary">{chapter.summary}</p>
          </div>
          <span className="chapter-section-count">
            {components.length} components
          </span>
        </div>
      )}

      {isVisible ? (
        viewMode === "grid" ? (
          <div className="components-card-grid">
            {components.map((item) => (
              <ComponentCard key={item.name} item={item} />
            ))}
          </div>
        ) : (
          <div className="components-list-table">
            {components.map((item) => (
              <div key={item.name} className="list-row-item">
                <span className="list-col-name">{item.display_name}</span>
                <span className="list-col-desc">{item.description}</span>
                <code className="list-col-type">{item.export_type}</code>
                <code className="list-col-cli">cargo ely add {item.name}</code>
              </div>
            ))}
          </div>
        )
      ) : (
        <div
          className="lazy-chapter-placeholder"
          style={{ height: Math.min(600, Math.max(160, Math.ceil(components.length / 3) * 140)) }}
        >
          <span className="placeholder-loading-text">Loading {chapter.title} ({components.length} components)...</span>
        </div>
      )}
    </section>
  );
}
