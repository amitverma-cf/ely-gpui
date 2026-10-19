"""Generates the full registry.json containing all 1280 components across all 44 chapters."""

import json
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STORIES_PATH = ROOT / "examples/gallery/stories.json"
TASKS_DIR = ROOT / "tasks"
SRC_DIR = ROOT / "src"
OUT_PATH = ROOT / "registry.json"

SLUG_MAP = {
    "collaboration": "collab",
    "monitoring": "dashboard",
    "a11y": "i18n",
    "data": "data_display",
}


def load_task_info():
    task_info = {}
    for f in sorted(TASKS_DIR.glob("ch*.md")):
        for line in f.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line.startswith("- [x] ") or line.startswith("- [ ] "):
                m = re.match(r"- \[[ x]\] (.*?)(?: — (.*)| → `?([\w:]+)`?|$)", line)
                if m:
                    names_part = m.group(1).strip()
                    desc = m.group(2) or ""
                    target = m.group(3) or ""
                    for name in names_part.split(" / "):
                        clean = re.sub(r"\s*\([^)]*\)", "", name).strip()
                        if clean:
                            key = re.sub(r"[^a-z0-9]", "", clean.lower())
                            task_info[key] = {
                                "original": clean,
                                "desc": desc.strip(),
                                "target": target.strip(),
                            }
    return task_info


def find_component_file(src_path, comp_name):
    clean = re.sub(r"[^a-zA-Z0-9]", "", comp_name)
    snake = re.sub(r"(?<!^)(?=[A-Z])", "_", clean).lower()

    if not src_path.is_dir():
        return "mod.rs", clean

    rs_files = [f for f in os.listdir(src_path) if f.endswith(".rs")]

    # 1. Direct file match: foo_bar.rs
    if f"{snake}.rs" in rs_files:
        return f"{snake}.rs", clean

    # 2. Search definitions in .rs files
    for f in rs_files:
        try:
            content = (src_path / f).read_text(encoding="utf-8", errors="ignore")
            if re.search(r"\b(struct|enum|fn|type)\s+" + re.escape(clean) + r"\b", content):
                return f, clean
        except Exception:
            pass

    # 3. Substring match
    for f in rs_files:
        try:
            content = (src_path / f).read_text(encoding="utf-8", errors="ignore")
            if clean in content:
                return f, clean
        except Exception:
            pass

    if "mod.rs" in rs_files:
        return "mod.rs", clean
    elif rs_files:
        return rs_files[0], clean
    return "mod.rs", clean


def main():
    stories_data = json.loads(STORIES_PATH.read_text(encoding="utf-8"))
    task_info = load_task_info()

    chapters = []
    components = []
    seen_components = set()

    # Pre-defined known dependencies
    known_deps = {
        "command_palette": ["badge", "input"],
        "dialog": ["button"],
        "confirm_dialog": ["button", "dialog"],
        "alert_dialog": ["dialog"],
        "prompt_dialog": ["dialog", "input"],
        "search_input": ["input", "icon"],
        "editor_tabs": ["tabs"],
    }

    for page in stories_data["pages"]:
        num = page["number"]
        slug = page["slug"]
        title = page["title"]
        summary = page["summary"]
        src_sub = SLUG_MAP.get(slug, slug)
        src_path = SRC_DIR / src_sub

        chapter_comp_names = []

        for story in page.get("stories", []):
            story_title = story.get("title", "")
            for comp_raw in story.get("components", []):
                comp_clean = re.sub(r"\s*\([^)]*\)", "", comp_raw).strip()
                if not comp_clean:
                    continue

                slug_name = re.sub(
                    r"(?<!^)(?=[A-Z])", "_", comp_clean.replace(" ", "")
                ).lower().replace("-", "_")

                if comp_clean in seen_components:
                    continue
                seen_components.add(comp_clean)
                chapter_comp_names.append(comp_clean)

                key = re.sub(r"[^a-z0-9]", "", comp_clean.lower())
                info = task_info.get(key, {})
                desc = info.get("desc", "")
                if not desc:
                    desc = f"{comp_clean} component in {title}"

                file_name, export_type = find_component_file(src_path, comp_clean)
                deps = known_deps.get(slug_name, [])

                components.append({
                    "name": slug_name,
                    "display_name": comp_clean,
                    "chapter": num,
                    "chapter_title": title,
                    "category": slug,
                    "source_dir": f"src/{src_sub}",
                    "file": file_name,
                    "export_type": export_type,
                    "description": desc,
                    "dependencies": deps,
                })

        chapters.append({
            "number": num,
            "name": slug,
            "title": title,
            "summary": summary,
            "source_dir": f"src/{src_sub}",
            "component_count": len(chapter_comp_names),
            "components": chapter_comp_names,
        })

    registry = {
        "$schema": "https://elygpui.com/schema/registry.json",
        "version": "1.0.0",
        "total_components": len(components),
        "total_chapters": len(chapters),
        "chapters": chapters,
        "components": components,
    }

    OUT_PATH.write_text(json.dumps(registry, indent=2), encoding="utf-8")
    print(
        f"[ok] Successfully wrote {OUT_PATH} with {len(components)} components across {len(chapters)} chapters."
    )


if __name__ == "__main__":
    main()
