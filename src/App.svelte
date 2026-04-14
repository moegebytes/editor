<script lang="ts">
  import dayjs from 'dayjs';
  import { SvelteMap, SvelteSet } from 'svelte/reactivity';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  import type {
    AppSettings,
    FlatEntry,
    GlossaryEntry,
    ProjectFiles,
    ProjectSettings,
    RecoveryEntry,
  } from './lib/types';
  import { type Command, UndoStack } from './lib/undo.svelte';
  import { useDebouncedValue } from './lib/debounce.svelte.js';
  import { exportProjectDialog } from './lib/dialogs';
  import { isText, isTranslated, isUntranslated, modKey } from './lib/utils';
  import { toast } from './lib/toast.svelte';
  import {
    checkRecovery,
    closeProject,
    confirmLine,
    createProject,
    deleteRecovery,
    exportProject,
    getAppSettings,
    importProject,
    loadRecovery,
    openProject,
    saveProject,
    saveTranslation,
    unconfirmLine,
    updateAppSettings,
    updateGlossary,
    updateProject,
    writeRecovery,
  } from './lib/ipc';

  import AboutDialog from './components/AboutDialog.svelte';
  import ContextMenu from './components/ui/ContextMenu.svelte';
  import DictionaryPanel from './components/DictionaryPanel.svelte';
  import EditorTable from './components/EditorTable.svelte';
  import GlossaryPanel from './components/GlossaryPanel.svelte';
  import FindReplaceBar from './components/FindReplaceBar.svelte';
  import GoToLineDialog from './components/GoToLineDialog.svelte';
  import ProjectHome from './components/ProjectHome.svelte';
  import RecoveryDialog from './components/RecoveryDialog.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import ToastContainer from './components/ui/ToastContainer.svelte';
  import Toolbar from './components/Toolbar.svelte';
  import UnsavedChangesDialog from './components/UnsavedChangesDialog.svelte';

  // Project
  let projectId: string | null = $state(null);
  let projectName: string | null = $state(null);
  let projectFiles: ProjectFiles = $state({ jp: '', en: '' });
  let confirmedLines = new SvelteSet<number>();
  let projectSettings: ProjectSettings = $state({});
  let appSettings: AppSettings = $state({ autoConfirmOnEnter: false, partialSearch: false, autoSaveIntervalSecs: 0 });
  let minAutoSaveIntervalSecs = $state(30);
  let glossary: GlossaryEntry[] = $state([]);
  let glossaryVisible = $state(false);
  let settingsVisible = $state(false);
  let goToLineVisible = $state(false);
  let aboutVisible = $state(false);

  // Editor
  let entries: FlatEntry[] = $state([]);
  let dirtyIndices = new SvelteMap<number, number>();
  const undoStack = new UndoStack();
  let modified = $state(false);
  let loading = $state(false);
  let saving = $state(false);
  let selectedIndex = $state(-1);

  // Dictionary
  let dictVisible = $state(false);
  let dictQuery = $state('');
  let dictQuerySeq = $state(0);

  // Filter
  let filterText = $state('');
  const debouncedFilter = useDebouncedValue(() => filterText, 150);
  let findReplaceVisible = $state(false);
  let findQuery = $state('');
  let findCaseSensitive = $state(false);
  let findMatchIndices: number[] = $state([]);
  let currentFindMatch = $state(-1);
  let toolbarRef: Toolbar = $state() as Toolbar;

  // Unsaved changes dialog
  let unsavedDialogVisible = $state(false);
  let pendingAction: (() => void) | null = null;
  let closingConfirmed = false;

  // Recovery dialog
  let recoveryDialogVisible = $state(false);
  let recoveryTimestamp = $state(0);
  let recoveryEntryCount = $state(0);
  let recoveryConfirmedDelta = $state(0);

  let stats = $derived.by(() => {
    let totalText = 0,
      translated = 0,
      confirmed = 0;
    for (const e of entries) {
      if (!isText(e)) continue;
      totalText++;
      if (e.enText) translated++;
      if (confirmedLines.has(e.index)) confirmed++;
    }
    return { totalText, translated, confirmed };
  });

  let hasProject = $derived(projectName !== null);

  getAppSettings().then((s) => {
    const { minAutoSaveIntervalSecs: min, ...settings } = s;
    appSettings = settings;
    minAutoSaveIntervalSecs = min;
  });

  getCurrentWindow().onCloseRequested((event) => {
    if (modified && !closingConfirmed) {
      event.preventDefault();
      guardUnsaved(() => {
        closingConfirmed = true;
        getCurrentWindow().close();
      });
    }
  });

  $effect(() => {
    void debouncedFilter.value;
    void findCaseSensitive;
    if (findQuery) computeFindMatches(findQuery);
  });

  function buildRecoveryEntries(): Record<string, RecoveryEntry> {
    const result: Record<string, RecoveryEntry> = {};
    for (const [idx, count] of dirtyIndices) {
      if (count <= 0) continue;
      const e = entries[idx];
      if (e) result[idx] = { enText: e.enText, notes: e.notes };
    }
    return result;
  }

  $effect(() => {
    const interval = appSettings.autoSaveIntervalSecs;
    if (!hasProject || interval === 0) return;
    const timer = setInterval(() => {
      if (modified) writeRecovery(buildRecoveryEntries()).catch(() => {});
    }, interval * 1000);
    return () => clearInterval(timer);
  });

  function guardUnsaved(action: () => void) {
    if (modified) {
      pendingAction = action;
      unsavedDialogVisible = true;
    } else {
      action();
    }
  }

  function handleUnsavedSave() {
    const action = pendingAction;
    pendingAction = null;
    handleSave().then(() => {
      if (!modified) action?.();
    });
  }

  function handleUnsavedDiscard() {
    const action = pendingAction;
    pendingAction = null;
    action?.();
  }

  function applyProject(proj: import('./lib/types').Project) {
    projectId = proj.id;
    projectName = proj.name;
    projectFiles = proj.files;
    confirmedLines.clear();
    for (const i of proj.confirmedLines) confirmedLines.add(i);
    projectSettings = proj.settings;
    glossary = proj.glossary ?? [];
    entries = proj.entries;
    dirtyIndices.clear();
    undoStack.clear();
    modified = false;
  }

  async function checkForRecovery(id: string) {
    const info = await checkRecovery(id);
    if (info) {
      recoveryTimestamp = info.timestamp;
      recoveryEntryCount = info.entryCount;
      recoveryConfirmedDelta = info.confirmedLineCount - confirmedLines.size;
      recoveryDialogVisible = true;
    }
  }

  async function handleRecoveryRestore() {
    if (!projectId) return;
    try {
      const data = await loadRecovery(projectId);
      for (const [idx, rec] of Object.entries(data.entries)) {
        const entry = entries[Number(idx)];
        if (entry) {
          entry.enText = rec.enText;
          entry.notes = rec.notes;
        }
      }
      confirmedLines.clear();
      for (const i of data.confirmedLines) confirmedLines.add(i);
      dirtyIndices.clear();
      modified = true;
      await deleteRecovery(projectId);
      toast.success('Changes restored from snapshot');
    } catch (e) {
      toast.error(`Failed to restore from snapshot: ${e}`);
    }
  }

  function handleRecoveryDiscard() {
    if (projectId) deleteRecovery(projectId).catch(() => {});
  }

  async function handleNewProject(name: string, jp: string, en: string) {
    try {
      loading = true;
      const proj = await createProject(name, { jp, en });
      applyProject(proj);
      await checkForRecovery(proj.id);
    } catch (e) {
      toast.error(`Failed to create project: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function handleImportProject(sourcePath: string, name: string, jp: string, en: string) {
    try {
      loading = true;
      const proj = await importProject(sourcePath, name, { jp, en });
      applyProject(proj);
      await checkForRecovery(proj.id);
    } catch (e) {
      toast.error(`Failed to import project: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function handleOpenProject(id: string) {
    try {
      loading = true;
      applyProject(await openProject(id));
      await checkForRecovery(id);
    } catch (e) {
      toast.error(`Failed to open project: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function handleUpdateProject(name: string, settings: ProjectSettings, newAppSettings: AppSettings) {
    if (!projectId) return;
    try {
      await updateProject(projectId, name, projectFiles, settings);
      await updateAppSettings(newAppSettings);
      projectName = name;
      projectSettings = settings;
      appSettings = newAppSettings;
      toast.success('Settings saved');
    } catch (e) {
      toast.error(`Failed to save settings: ${e}`);
    }
  }

  async function handleSaveGlossary(newGlossary: GlossaryEntry[]) {
    try {
      await updateGlossary(newGlossary);
      glossary = newGlossary;
      toast.success('Glossary saved');
    } catch (e) {
      toast.error(`Failed to save glossary: ${e}`);
    }
  }

  async function handleSave() {
    if (!hasProject) return;
    try {
      saving = true;
      await saveTranslation(entries);
      await saveProject();
      dirtyIndices.clear();
      modified = false;
      if (projectId) deleteRecovery(projectId).catch(() => {});
      toast.success('Project saved');
    } catch (e) {
      toast.error(`Failed to save: ${e}`);
    } finally {
      saving = false;
    }
  }

  async function handleExport() {
    const date = dayjs().format('YYYY-MM-DD HH-mm-ss');
    const path = await exportProjectDialog(`${projectName} - ${date}.zip`);
    if (!path) return;
    try {
      await exportProject(path);
      toast.success('Project exported');
    } catch (e) {
      toast.error(`Failed to export: ${e}`);
    }
  }

  function doCloseProject() {
    if (projectId) deleteRecovery(projectId).catch(() => {});
    closeProject();
    projectId = null;
    projectName = null;
    projectFiles = { jp: '', en: '' };
    entries = [];
    dirtyIndices.clear();
    undoStack.clear();
    confirmedLines.clear();
    projectSettings = {};
    glossary = [];
    glossaryVisible = false;
    modified = false;
    selectedIndex = -1;
    filterText = '';
    findReplaceVisible = false;
    findQuery = '';
    findCaseSensitive = false;
    findMatchIndices = [];
    currentFindMatch = -1;
  }

  function handleCloseProject() {
    guardUnsaved(doCloseProject);
  }

  function incrementDirty(index: number) {
    dirtyIndices.set(index, (dirtyIndices.get(index) ?? 0) + 1);
  }

  function decrementDirty(index: number) {
    dirtyIndices.set(index, (dirtyIndices.get(index) ?? 0) - 1);
  }

  function handleEnTextChange(index: number, newText: string) {
    const oldText = entries[index].enText;
    entries[index].enText = newText;
    const coalesced = undoStack.coalesceText(index, oldText, newText);
    if (!coalesced) incrementDirty(index);
    modified = true;
  }

  function handleNotesChange(index: number, notes: string[]) {
    const oldNotes = [...entries[index].notes];
    entries[index].notes = notes;
    undoStack.push({ kind: 'editNotes', index, oldNotes, newNotes: [...notes] });
    incrementDirty(index);
    modified = true;
  }

  async function handleToggleConfirm(index: number, grouped = false) {
    const wasConfirmed = confirmedLines.has(index);
    if (wasConfirmed) {
      confirmedLines.delete(index);
      try {
        await unconfirmLine(index);
      } catch (e) {
        confirmedLines.add(index);
        toast.error(`Failed to unconfirm line: ${e}`);
        return;
      }
    } else {
      confirmedLines.add(index);
      try {
        await confirmLine(index);
      } catch (e) {
        confirmedLines.delete(index);
        toast.error(`Failed to confirm line: ${e}`);
        return;
      }
    }
    const cmd: Command = wasConfirmed ? { kind: 'unconfirm', index } : { kind: 'confirm', index };
    if (grouped) {
      undoStack.groupWithLast(cmd);
    } else {
      undoStack.push(cmd);
    }
    modified = true;
  }

  function jumpToNext(predicate: (e: FlatEntry) => boolean) {
    const start = selectedIndex >= 0 ? selectedIndex : -1;
    for (let i = start + 1; i < entries.length; i++) {
      if (predicate(entries[i])) {
        selectedIndex = entries[i].index;
        return;
      }
    }
    for (let i = 0; i <= start; i++) {
      if (predicate(entries[i])) {
        selectedIndex = entries[i].index;
        return;
      }
    }
  }

  function jumpToNextUntranslated() {
    jumpToNext(isUntranslated);
  }

  function jumpToNextUnconfirmed() {
    jumpToNext((e) => isTranslated(e) && !confirmedLines.has(e.index));
  }

  function confirmToggleCurrent() {
    if (selectedIndex >= 0) {
      handleToggleConfirm(selectedIndex);
    }
  }

  function handleGoToLine(line: number) {
    const idx = line - 1;
    if (idx >= 0 && idx < entries.length) {
      selectedIndex = entries[idx].index;
    }
  }

  function handleJpSelect(text: string) {
    dictQuery = text;
    dictQuerySeq++;
    dictVisible = true;
  }

  function computeFindMatches(query: string) {
    findQuery = query;
    if (!query) {
      findMatchIndices = [];
      currentFindMatch = -1;
      return;
    }
    const needle = findCaseSensitive ? query : query.toLowerCase();
    const fLower = debouncedFilter.value.toLowerCase();
    const matches: number[] = [];
    for (const entry of entries) {
      if (!isText(entry)) continue;
      const enText = entry.enText ?? '';
      const enLower = findCaseSensitive && !fLower ? '' : enText.toLowerCase();
      if (fLower) {
        const jpLower = (entry.jpText ?? '').toLowerCase();
        if (!jpLower.includes(fLower) && !enLower.includes(fLower)) continue;
      }
      const haystack = findCaseSensitive ? enText : enLower;
      if (haystack.includes(needle)) matches.push(entry.index);
    }
    findMatchIndices = matches;
    currentFindMatch = matches.length > 0 ? 0 : -1;
  }

  function findNext() {
    if (findMatchIndices.length === 0) return;
    currentFindMatch = (currentFindMatch + 1) % findMatchIndices.length;
    selectedIndex = findMatchIndices[currentFindMatch];
  }

  function findPrev() {
    if (findMatchIndices.length === 0) return;
    currentFindMatch = (currentFindMatch - 1 + findMatchIndices.length) % findMatchIndices.length;
    selectedIndex = findMatchIndices[currentFindMatch];
  }

  function normalize(s: string): string {
    return findCaseSensitive ? s : s.toLowerCase();
  }

  function replaceCurrent(replacement: string) {
    if (currentFindMatch < 0 || !findQuery) return;
    const idx = findMatchIndices[currentFindMatch];
    const entry = entries[idx];
    if (!entry.enText) return;
    const pos = normalize(entry.enText).indexOf(normalize(findQuery));
    if (pos < 0) return;
    const oldText = entry.enText;
    entries[idx].enText = entry.enText.substring(0, pos) + replacement + entry.enText.substring(pos + findQuery.length);
    undoStack.push({ kind: 'editText', index: idx, oldText, newText: entries[idx].enText });
    incrementDirty(idx);
    modified = true;
    computeFindMatches(findQuery);
  }

  function replaceAll(query: string, replacement: string) {
    if (!query) return;
    const needle = normalize(query);
    const matchSet = new Set(findMatchIndices);
    const commands: Command[] = [];
    for (const entry of entries) {
      if (!matchSet.has(entry.index) || !entry.enText) continue;
      const en = entry.enText;
      const haystack = normalize(en);
      let result = '';
      let i = 0;
      while (i < en.length) {
        const pos = haystack.indexOf(needle, i);
        if (pos < 0) {
          result += en.substring(i);
          break;
        }
        result += en.substring(i, pos) + replacement;
        i = pos + query.length;
      }
      if (result !== en) {
        commands.push({ kind: 'editText', index: entry.index, oldText: en, newText: result });
        entry.enText = result;
        incrementDirty(entry.index);
      }
    }
    if (commands.length > 0) {
      undoStack.push(commands.length === 1 ? commands[0] : commands);
      modified = true;
    }
    computeFindMatches(query);
  }

  async function applyEntry(commands: Command[], reverse: boolean) {
    const ipcCalls: Promise<void>[] = [];
    for (const cmd of commands) {
      if (cmd.kind === 'editText') {
        entries[cmd.index].enText = reverse ? cmd.oldText : cmd.newText;
        (reverse ? decrementDirty : incrementDirty)(cmd.index);
      } else if (cmd.kind === 'editNotes') {
        entries[cmd.index].notes = reverse ? [...cmd.oldNotes] : [...cmd.newNotes];
        (reverse ? decrementDirty : incrementDirty)(cmd.index);
      } else if (cmd.kind === 'confirm') {
        if (reverse) {
          confirmedLines.delete(cmd.index);
          ipcCalls.push(unconfirmLine(cmd.index));
        } else {
          confirmedLines.add(cmd.index);
          ipcCalls.push(confirmLine(cmd.index));
        }
      } else if (cmd.kind === 'unconfirm') {
        if (reverse) {
          confirmedLines.add(cmd.index);
          ipcCalls.push(confirmLine(cmd.index));
        } else {
          confirmedLines.delete(cmd.index);
          ipcCalls.push(unconfirmLine(cmd.index));
        }
      }
    }
    if (ipcCalls.length > 0) await Promise.all(ipcCalls);
  }

  async function handleUndo() {
    const entry = undoStack.popUndo();
    if (!entry) return;
    try {
      const commands = Array.isArray(entry) ? [...entry].reverse() : [entry];
      await applyEntry(commands, true);
      modified = true;
    } catch (e) {
      toast.error(`Undo failed: ${e}`);
    }
  }

  async function handleRedo() {
    const entry = undoStack.popRedo();
    if (!entry) return;
    try {
      const commands = Array.isArray(entry) ? entry : [entry];
      await applyEntry(commands, false);
      modified = true;
    } catch (e) {
      toast.error(`Redo failed: ${e}`);
    }
  }

  function handleKeydownGlobal(e: KeyboardEvent) {
    if (e.key === 'F3' || e.key === 'F5' || e.key === 'F7') {
      e.preventDefault();
      return;
    }

    if (!modKey(e)) return;

    switch (e.key) {
      case 'f':
        toolbarRef?.focusFilter();
        break;
      case 'h':
        findReplaceVisible = !findReplaceVisible;
        break;
      case 'd':
        dictVisible = !dictVisible;
        break;
      case 'g':
        goToLineVisible = true;
        break;
      case 'z':
        handleUndo();
        break;
      case 'y':
        handleRedo();
        break;
      case 'r':
      case 'j':
      case 'p':
        break;
      default:
        return;
    }
    e.preventDefault();
  }
</script>

<svelte:window onkeydowncapture={handleKeydownGlobal} />

<div class="app">
  {#if !hasProject}
    <ProjectHome
      onNewProject={handleNewProject}
      onImportProject={handleImportProject}
      onOpenProject={handleOpenProject}
      {loading}
    />
  {:else if glossaryVisible}
    <GlossaryPanel {glossary} onBack={() => (glossaryVisible = false)} onSave={handleSaveGlossary} />
  {:else if settingsVisible}
    <SettingsView
      projectName={projectName ?? ''}
      files={projectFiles}
      bind:settings={projectSettings}
      bind:appSettings
      {minAutoSaveIntervalSecs}
      onBack={() => (settingsVisible = false)}
      onSave={handleUpdateProject}
    />
  {:else}
    <Toolbar
      bind:this={toolbarRef}
      onSave={handleSave}
      onExport={handleExport}
      onCloseProject={handleCloseProject}
      onOpenDict={() => (dictVisible = true)}
      onOpenGlossary={() => (glossaryVisible = true)}
      onToggleFindReplace={() => (findReplaceVisible = !findReplaceVisible)}
      onJumpUntranslated={jumpToNextUntranslated}
      onJumpUnconfirmed={jumpToNextUnconfirmed}
      onConfirmToggle={confirmToggleCurrent}
      onUndo={handleUndo}
      onRedo={handleRedo}
      undoDisabled={!undoStack.canUndo}
      redoDisabled={!undoStack.canRedo}
      onGoToLine={() => (goToLineVisible = true)}
      onOpenSettings={() => (settingsVisible = true)}
      onAbout={() => (aboutVisible = true)}
      {projectName}
      saveDisabled={!modified}
      bind:filterText
    />

    <div class="main-area">
      <DictionaryPanel query={dictQuery} querySeq={dictQuerySeq} bind:visible={dictVisible} />

      <div class="editor-area">
        <EditorTable
          {entries}
          onEnTextChange={handleEnTextChange}
          onToggleConfirm={handleToggleConfirm}
          onNotesChange={handleNotesChange}
          onJumpNextUnconfirmed={jumpToNextUnconfirmed}
          {confirmedLines}
          {dirtyIndices}
          autoConfirmOnEnter={appSettings.autoConfirmOnEnter}
          bind:selectedIndex
          onSave={handleSave}
          onJpSelect={handleJpSelect}
          {glossary}
          filterText={debouncedFilter.value}
          {findQuery}
          {findMatchIndices}
          {currentFindMatch}
        />
      </div>
    </div>

    <FindReplaceBar
      bind:visible={findReplaceVisible}
      bind:caseSensitive={findCaseSensitive}
      onFind={computeFindMatches}
      onFindNext={findNext}
      onFindPrev={findPrev}
      onReplace={replaceCurrent}
      onReplaceAll={replaceAll}
      matchCount={findMatchIndices.length}
      currentMatch={currentFindMatch}
    />

    <StatusBar {modified} {saving} {stats} />
  {/if}
</div>

<ContextMenu />
<ToastContainer />

<GoToLineDialog bind:visible={goToLineVisible} maxLine={entries.length} onGo={handleGoToLine} />

<AboutDialog
  bind:visible={aboutVisible}
  {projectName}
  stats={hasProject ? stats : null}
  {appSettings}
  projectSettings={hasProject ? projectSettings : null}
/>

<UnsavedChangesDialog bind:visible={unsavedDialogVisible} onSave={handleUnsavedSave} onDiscard={handleUnsavedDiscard} />

<RecoveryDialog
  bind:visible={recoveryDialogVisible}
  timestamp={recoveryTimestamp}
  entryCount={recoveryEntryCount}
  confirmedDelta={recoveryConfirmedDelta}
  onRestore={handleRecoveryRestore}
  onDiscard={handleRecoveryDiscard}
/>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .main-area {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }

  .editor-area {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
</style>
