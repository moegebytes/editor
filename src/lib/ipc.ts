import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type {
  EnvironmentInfo,
  FlatEntry,
  ImportPreview,
  KanjiEntry,
  LookupResult,
  Project,
  ProjectFiles,
  ProjectInfo,
  ProjectSettings,
  RecentProject,
  WiktResult,
} from './types';

export async function saveTranslation(entries: FlatEntry[]): Promise<void> {
  return invoke('save_translation', { entries });
}

const STRINGS_FILTER = {
  name: 'Strings files',
  extensions: ['strings', 'txt'],
};

export async function openFileDialog(defaultPath?: string): Promise<string | null> {
  return open({ filters: [STRINGS_FILTER], defaultPath });
}

const PROJECT_FILTER = {
  name: 'Project files',
  extensions: ['json'],
};

export async function exportProjectDialog(): Promise<string | null> {
  return save({ filters: [PROJECT_FILTER] });
}

export async function lookupJmdict(query: string): Promise<LookupResult> {
  return invoke('lookup_jmdict', { query });
}

export async function lookupKanji(ch: string): Promise<KanjiEntry | null> {
  return invoke('lookup_kanji', { ch });
}

export async function lookupWiktionary(term: string): Promise<WiktResult> {
  return invoke('lookup_wiktionary', { term });
}

export async function createProject(name: string, files: ProjectFiles): Promise<Project> {
  return invoke('create_project', { name, files });
}

export async function openProject(id: string): Promise<Project> {
  return invoke('open_project', { id });
}

export async function saveProject(): Promise<void> {
  return invoke('save_project');
}

export async function confirmLine(index: number): Promise<void> {
  return invoke('confirm_line', { index });
}

export async function unconfirmLine(index: number): Promise<void> {
  return invoke('unconfirm_line', { index });
}

export async function listRecentProjects(): Promise<RecentProject[]> {
  return invoke('list_recent_projects');
}

export async function listAllProjects(): Promise<RecentProject[]> {
  return invoke('list_all_projects');
}

export async function removeRecentProject(id: string): Promise<void> {
  return invoke('remove_recent_project', { id });
}

export async function deleteProject(id: string): Promise<void> {
  return invoke('delete_project', { id });
}

export async function exportProject(destPath: string): Promise<void> {
  return invoke('export_project', { destPath });
}

export async function getProjectInfo(id: string): Promise<ProjectInfo> {
  return invoke('get_project_info', { id });
}

export async function updateProject(
  id: string,
  name: string,
  files: ProjectFiles,
  settings: ProjectSettings,
): Promise<void> {
  return invoke('update_project', { id, name, files, settings });
}

export async function importProjectDialog(): Promise<string | null> {
  return open({ filters: [PROJECT_FILTER] });
}

export async function previewImport(sourcePath: string): Promise<ImportPreview> {
  return invoke('preview_import', { sourcePath });
}

export async function importProject(sourcePath: string, name: string, files: ProjectFiles): Promise<Project> {
  return invoke('import_project', { sourcePath, name, files });
}

export async function openAppDir(): Promise<void> {
  return invoke('open_app_dir');
}

export async function getEnvironmentInfo(): Promise<EnvironmentInfo> {
  return invoke('get_environment_info');
}
