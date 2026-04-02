<script lang="ts">
  import type { FlatEntry, ProjectFiles, ProjectSettings } from './lib/types';
  import { isText, isUntranslated, isTranslated } from './lib/utils';
  import {
    confirmLine,
    createProject,
    exportProject,
    exportProjectDialog,
    importProject,
    openProject,
    updateProject,
    saveTranslation,
    saveProject,
    unconfirmLine,
  } from './lib/ipc';
  import Toolbar from './components/Toolbar.svelte';
  import EditorTable from './components/EditorTable.svelte';
  import FindReplaceBar from './components/FindReplaceBar.svelte';
  import DictionaryPanel from './components/DictionaryPanel.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import ProjectHome from './components/ProjectHome.svelte';
  import ContextMenu from './components/ui/ContextMenu.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import GoToLineDialog from './components/GoToLineDialog.svelte';
  import AboutDialog from './components/AboutDialog.svelte';
  import UnsavedChangesDialog from './components/UnsavedChangesDialog.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { toast } from './lib/toast.svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import ToastContainer from './components/ui/ToastContainer.svelte';

  // Project state
  let projectId: string | null = $state(null);
  let projectName: string | null = $state(null);
  let projectFiles: ProjectFiles = $state({ jp: '', en: '' });
  let confirmedLines = new SvelteSet<number>();
  let projectSettings: ProjectSettings = $state({ autoConfirmOnEnter: false });
  let settingsVisible = $state(false);
  let goToLineVisible = $state(false);
  let aboutVisible = $state(false);

  // Editor state
  let entries: FlatEntry[] = $state([]);
  let modified = $state(false);
  let loading = $state(false);
  let saving = $state(false);
  let selectedIndex = $state(-1);

  // Dictionary state
  let dictVisible = $state(false);
  let dictQuery = $state('');

  // Filter state
  let filterText = $state('');
  let findReplaceVisible = $state(false);
  let findQuery = $state('');
  let findMatchIndices: number[] = $state([]);
  let currentFindMatch = $state(-1);
  let toolbarRef: Toolbar = $state() as Toolbar;

  // Unsaved changes dialog
  let unsavedDialogVisible = $state(false);
  let pendingAction: (() => void) | null = null;

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

  let stats = $derived({
    totalText: entries.filter(isText).length,
    translated: entries.filter(isTranslated).length,
    confirmed: entries.filter((e) => isText(e) && confirmedLines.has(e.index)).length,
  });

  let hasProject = $derived(projectName !== null);

  function applyProject(proj: import('./lib/types').Project) {
    projectId = proj.id;
    projectName = proj.name;
    projectFiles = proj.files;
    confirmedLines.clear();
    for (const i of proj.confirmedLines) confirmedLines.add(i);
    projectSettings = proj.settings;
    entries = proj.entries;
    modified = false;
  }

  async function handleNewProject(name: string, jp: string, en: string) {
    try {
      loading = true;
      applyProject(await createProject(name, { jp, en }));
    } catch (e) {
      toast.error(`Failed to create project: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function handleImportProject(sourcePath: string, name: string, jp: string, en: string) {
    try {
      loading = true;
      applyProject(await importProject(sourcePath, name, { jp, en }));
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
    } catch (e) {
      toast.error(`Failed to open project: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function handleUpdateProject(name: string, settings: ProjectSettings) {
    if (!projectId) return;
    try {
      await updateProject(projectId, name, projectFiles, settings);
      projectName = name;
      projectSettings = settings;
      toast.success('Settings saved');
    } catch (e) {
      toast.error(`Failed to save settings: ${e}`);
    }
  }

  async function handleSave() {
    if (!hasProject) return;
    try {
      saving = true;
      await saveTranslation(entries);
      await saveProject();
      modified = false;
      toast.success('Project saved');
    } catch (e) {
      toast.error(`Failed to save: ${e}`);
    } finally {
      saving = false;
    }
  }

  async function handleExport() {
    const path = await exportProjectDialog();
    if (!path) return;
    try {
      await exportProject(path);
      toast.success('Project exported');
    } catch (e) {
      toast.error(`Failed to export: ${e}`);
    }
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

  function handleEnTextChange(index: number, newText: string) {
    entries[index].enText = newText;
    modified = true;
  }

  function handleNotesChange(index: number, notes: string[]) {
    entries[index].notes = notes;
    modified = true;
  }

  async function handleToggleConfirm(index: number) {
    if (confirmedLines.has(index)) {
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
    modified = true;
  }

  let dictQuerySeq = $state(0);

  function handleJpSelect(text: string) {
    dictQuery = text;
    dictQuerySeq++;
    dictVisible = true;
  }

  function doCloseProject() {
    projectId = null;
    projectName = null;
    projectFiles = { jp: '', en: '' };
    entries = [];
    confirmedLines.clear();
    projectSettings = { autoConfirmOnEnter: false };
    modified = false;
    selectedIndex = -1;
    filterText = '';
    findReplaceVisible = false;
    findQuery = '';
    findMatchIndices = [];
    currentFindMatch = -1;
  }

  function handleCloseProject() {
    guardUnsaved(doCloseProject);
  }

  // Intercept window close to warn about unsaved changes
  let closingConfirmed = false;

  getCurrentWindow().onCloseRequested((event) => {
    if (modified && !closingConfirmed) {
      event.preventDefault();
      guardUnsaved(() => {
        closingConfirmed = true;
        getCurrentWindow().close();
      });
    }
  });

  // Find/replace logic
  function computeFindMatches(query: string) {
    findQuery = query;
    if (!query) {
      findMatchIndices = [];
      currentFindMatch = -1;
      return;
    }
    const lower = query.toLowerCase();
    const fLower = filterText.toLowerCase();
    const matches: number[] = [];
    for (const entry of entries) {
      if (!isText(entry)) continue;
      if (fLower) {
        const jp = (entry.jpText ?? '').toLowerCase();
        const en = (entry.enText ?? '').toLowerCase();
        if (!jp.includes(fLower) && !en.includes(fLower)) continue;
      }
      const en = (entry.enText ?? '').toLowerCase();
      if (en.includes(lower)) matches.push(entry.index);
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

  function replaceCurrent(replacement: string) {
    if (currentFindMatch < 0 || !findQuery) return;
    const idx = findMatchIndices[currentFindMatch];
    const entry = entries[idx];
    if (!entry.enText) return;
    const lower = entry.enText.toLowerCase();
    const pos = lower.indexOf(findQuery.toLowerCase());
    if (pos < 0) return;
    entries[idx].enText = entry.enText.substring(0, pos) + replacement + entry.enText.substring(pos + findQuery.length);
    modified = true;
    computeFindMatches(findQuery);
  }

  function replaceAll(query: string, replacement: string) {
    if (!query) return;
    const lower = query.toLowerCase();
    let count = 0;
    for (const entry of entries) {
      if (entry.entryType !== 'text' || !entry.enText) continue;
      const en = entry.enText;
      let result = '';
      let i = 0;
      const enLower = en.toLowerCase();
      while (i < en.length) {
        const pos = enLower.indexOf(lower, i);
        if (pos < 0) {
          result += en.substring(i);
          break;
        }
        result += en.substring(i, pos) + replacement;
        i = pos + query.length;
        count++;
      }
      if (result !== en) entry.enText = result;
    }
    if (count > 0) modified = true;
    computeFindMatches(query);
  }

  function handleKeydownGlobal(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'f') {
      e.preventDefault();
      toolbarRef?.focusFilter();
      return;
    }
    if (e.ctrlKey && e.key === 'h') {
      e.preventDefault();
      findReplaceVisible = !findReplaceVisible;
      return;
    }
    if (e.ctrlKey && e.key === 'd') {
      e.preventDefault();
      dictVisible = !dictVisible;
      return;
    }
    if (e.ctrlKey && e.key === 'g') {
      e.preventDefault();
      goToLineVisible = true;
      return;
    }
  }

  function handleGoToLine(line: number) {
    const idx = line - 1;
    if (idx >= 0 && idx < entries.length) {
      selectedIndex = entries[idx].index;
    }
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
  {:else if settingsVisible}
    <SettingsView
      projectName={projectName ?? ''}
      files={projectFiles}
      bind:settings={projectSettings}
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
      onToggleFindReplace={() => (findReplaceVisible = !findReplaceVisible)}
      onJumpUntranslated={jumpToNextUntranslated}
      onJumpUnconfirmed={jumpToNextUnconfirmed}
      onConfirmToggle={confirmToggleCurrent}
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
          autoConfirmOnEnter={projectSettings.autoConfirmOnEnter}
          bind:selectedIndex
          onSave={handleSave}
          onJpSelect={handleJpSelect}
          {filterText}
          {findQuery}
          {findMatchIndices}
          {currentFindMatch}
        />
      </div>
    </div>

    <FindReplaceBar
      bind:visible={findReplaceVisible}
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

<AboutDialog bind:visible={aboutVisible} {projectName} stats={hasProject ? stats : null} />

<UnsavedChangesDialog bind:visible={unsavedDialogVisible} onSave={handleUnsavedSave} onDiscard={handleUnsavedDiscard} />

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
