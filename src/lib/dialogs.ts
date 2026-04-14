import { open, save } from '@tauri-apps/plugin-dialog';

export async function openFileDialog(defaultPath?: string): Promise<string | null> {
  return open({ filters: [{ name: 'Strings files', extensions: ['strings', 'txt'] }], defaultPath });
}

export async function exportProjectDialog(defaultName?: string): Promise<string | null> {
  return save({ filters: [{ name: 'Project archive', extensions: ['zip'] }], defaultPath: defaultName });
}

export async function importProjectDialog(): Promise<string | null> {
  return open({ filters: [{ name: 'Project files', extensions: ['json'] }] });
}
