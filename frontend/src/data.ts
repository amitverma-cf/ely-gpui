import manifest from "../../examples/gallery/stories.json";

export interface Story {
  title: string;
  /** The first component alone, as pages show it. */
  name: string;
  slug: string;
  components: string[];
}

export interface Chapter {
  number: number;
  slug: string;
  title: string;
  summary: string;
  stories: Story[];
}

export const chapters: Chapter[] = manifest.pages.map((c) => ({
  ...c,
  stories: c.stories.map((s) => ({ ...s, name: s.title.split(/ \/ | · | → | \(/)[0].trim() })),
}));

export const componentCount = new Set(chapters.flatMap((c) => c.stories.flatMap((s) => s.components))).size;

/** Every story in reading order, for paging and search. */
export const all = chapters.flatMap((chapter) => chapter.stories.map((story) => ({ chapter, story })));

/** Chapters by title, else their stories by title or component. */
export function search(query: string): Chapter[] {
  const q = query.trim().toLowerCase();
  if (!q) return chapters;
  return chapters.flatMap((c) => {
    if (c.title.toLowerCase().includes(q)) return [c];
    const stories = c.stories.filter((s) => s.title.toLowerCase().includes(q) || s.components.some((n) => n.toLowerCase().includes(q)));
    return stories.length ? [{ ...c, stories }] : [];
  });
}

import registryManifest from "../../registry.json";

export interface RegistryComponent {
  name: string;
  display_name: string;
  chapter: number;
  chapter_title: string;
  category: string;
  source_dir: string;
  file: string;
  export_type: string;
  description: string;
  dependencies: string[];
}

export interface RegistryChapter {
  number: number;
  name: string;
  title: string;
  summary: string;
  source_dir: string;
  component_count: number;
  components: string[];
}

export const registryComponents: RegistryComponent[] = registryManifest.components;
export const registryChapters: RegistryChapter[] = registryManifest.chapters;

export function getStoryUrl(chapterNumber: number, componentName: string): { page: string; story: string } {
  const ch = chapters.find((c) => c.number === chapterNumber);
  if (!ch) return { page: "primitives", story: "box-view" };
  const s = ch.stories.find((st) => st.components.includes(componentName)) || ch.stories[0];
  return { page: ch.slug, story: s ? s.slug : ch.stories[0].slug };
}

export const REPO = "https://github.com/amitverma-cf/ely-gpui";
export const ORIGINAL_REPO = "https://git.zacharyzhang.com/ZacharyZhang-NY/Ely-GPUI-Components";

export const pad = (n: number) => String(n).padStart(2, "0");
