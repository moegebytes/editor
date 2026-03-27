import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  FlatEntry,
  ImportPreview,
  KanjiEntry,
  LookupResult,
  Project,
  ProjectFiles,
  ProjectSettings,
  RecentProject,
} from "./types";

export async function saveEnFile(entries: FlatEntry[]): Promise<void> {
  return invoke("save_en_file", { entries });
}

const STRINGS_FILTER = {
  name: "Strings files",
  extensions: ["strings", "txt"],
};

export async function openFileDialog(): Promise<string | null> {
  return open({ filters: [STRINGS_FILTER] });
}

const PROJECT_FILTER = {
  name: "Project files",
  extensions: ["json"],
};

export async function exportProjectDialog(): Promise<string | null> {
  return save({ filters: [PROJECT_FILTER] });
}

export async function lookupWord(query: string): Promise<LookupResult> {
  return invoke("lookup_word", { query });
}

export async function lookupKanji(ch: string): Promise<KanjiEntry | null> {
  return invoke("lookup_kanji", { ch });
}

export async function createProject(name: string, files: ProjectFiles): Promise<Project> {
  return invoke("create_project", { name, files });
}

export async function openProject(id: string): Promise<Project> {
  return invoke("open_project", { id });
}

export async function saveProject(): Promise<void> {
  return invoke("save_project");
}

export async function confirmLine(index: number): Promise<void> {
  return invoke("confirm_line", { index });
}

export async function unconfirmLine(index: number): Promise<void> {
  return invoke("unconfirm_line", { index });
}

export async function listRecentProjects(): Promise<RecentProject[]> {
  return invoke("list_recent_projects");
}

export async function listAllProjects(): Promise<RecentProject[]> {
  return invoke("list_all_projects");
}

export async function removeRecentProject(id: string): Promise<void> {
  return invoke("remove_recent_project", { id });
}

export async function deleteProject(id: string): Promise<void> {
  return invoke("delete_project", { id });
}

export async function exportProject(destPath: string): Promise<void> {
  return invoke("export_project", { destPath });
}

export async function renameProject(name: string): Promise<void> {
  return invoke("rename_project", { name });
}

export async function importProjectDialog(): Promise<string | null> {
  return open({ filters: [PROJECT_FILTER] });
}

export async function previewImport(sourcePath: string): Promise<ImportPreview> {
  return invoke("preview_import", { sourcePath });
}

export async function importProject(
  sourcePath: string,
  name: string,
  files: ProjectFiles,
): Promise<Project> {
  return invoke("import_project", { sourcePath, name, files });
}

export async function updateProjectSettings(settings: ProjectSettings): Promise<void> {
  return invoke("update_project_settings", { settings });
}
