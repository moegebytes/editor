<script lang="ts">
  import type { FlatEntry, ProjectSettings } from "./lib/types";
  import { isText, isUntranslated, isTranslated } from "./lib/utils";
  import {
    confirmLine,
    createProject,
    exportProject,
    exportProjectDialog,
    importProject,
    openProject,
    renameProject,
    saveEnFile,
    saveProject,
    unconfirmLine,
    updateProjectSettings,
  } from "./lib/ipc";
  import Toolbar from "./components/Toolbar.svelte";
  import EditorTable from "./components/EditorTable.svelte";
  import FindReplaceBar from "./components/FindReplaceBar.svelte";
  import DictionaryPanel from "./components/DictionaryPanel.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import ProjectHome from "./components/ProjectHome.svelte";
  import ContextMenu from "./components/ui/ContextMenu.svelte";
  import SettingsView from "./components/SettingsView.svelte";
  import GoToLineDialog from "./components/GoToLineDialog.svelte";
  import UnsavedChangesDialog from "./components/UnsavedChangesDialog.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { XIcon } from "@lucide/svelte";

  // Project state
  let projectName: string | null = $state(null);
  let confirmedLines: Set<number> = $state(new Set());
  let projectSettings: ProjectSettings = $state({ autoConfirmOnEnter: false });
  let settingsVisible = $state(false);
  let goToLineVisible = $state(false);

  // Editor state
  let entries: FlatEntry[] = $state([]);
  let modified = $state(false);
  let loading = $state(false);
  let saving = $state(false);
  let error: string | null = $state(null);
  let selectedIndex = $state(-1);

  // Dictionary state
  let dictVisible = $state(false);
  let dictWidth = $state(320);
  let dictQuery = $state("");

  // Filter state
  let filterText = $state("");
  let findReplaceVisible = $state(false);
  let findQuery = $state("");
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
    confirmed: entries.filter(
      (e) => isText(e) && confirmedLines.has(e.index),
    ).length,
  });

  let hasProject = $derived(projectName !== null);

  function applyProject(proj: import("./lib/types").Project) {
    projectName = proj.name;
    confirmedLines = new Set(proj.confirmedLines);
    projectSettings = proj.settings;
    entries = proj.entries;
    modified = false;
  }

  async function handleNewProject(name: string, jp: string, en: string) {
    try {
      error = null;
      loading = true;
      applyProject(await createProject(name, { jp, en }));
    } catch (e) {
      error = `Failed to create project: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleImportProject(sourcePath: string, name: string, jp: string, en: string) {
    try {
      error = null;
      loading = true;
      applyProject(await importProject(sourcePath, name, { jp, en }));
    } catch (e) {
      error = `Failed to import project: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleOpenProject(id: string) {
    try {
      error = null;
      loading = true;
      applyProject(await openProject(id));
    } catch (e) {
      error = `Failed to open project: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleRename(name: string) {
    try {
      await renameProject(name);
      projectName = name;
    } catch (e) {
      error = `Failed to rename project: ${e}`;
    }
  }

  async function handleSaveSettings(settings: ProjectSettings) {
    try {
      await updateProjectSettings(settings);
    } catch (e) {
      error = `Failed to save settings: ${e}`;
    }
  }

  async function handleSave() {
    if (!hasProject) return;
    try {
      error = null;
      saving = true;
      await saveEnFile(entries);
      await saveProject();
      modified = false;
    } catch (e) {
      error = `Failed to save: ${e}`;
    } finally {
      saving = false;
    }
  }

  async function handleExport() {
    const path = await exportProjectDialog();
    if (!path) return;
    try {
      error = null;
      await exportProject(path);
    } catch (e) {
      error = `Failed to export: ${e}`;
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
    const next = new Set(confirmedLines);
    if (next.has(index)) {
      next.delete(index);
      await unconfirmLine(index);
    } else {
      next.add(index);
      await confirmLine(index);
    }
    confirmedLines = next;
    modified = true;
  }

  let dictQuerySeq = $state(0);

  function handleJpSelect(text: string) {
    dictQuery = text;
    dictQuerySeq++;
    dictVisible = true;
  }

  function doCloseProject() {
    projectName = null;
    entries = [];
    confirmedLines = new Set();
    projectSettings = { autoConfirmOnEnter: false };
    modified = false;
    selectedIndex = -1;
    filterText = "";
    findReplaceVisible = false;
    findQuery = "";
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
        const jp = (entry.jpText ?? "").toLowerCase();
        const en = (entry.enText ?? "").toLowerCase();
        if (!jp.includes(fLower) && !en.includes(fLower)) continue;
      }
      const en = (entry.enText ?? "").toLowerCase();
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
      if (entry.entryType !== "text" || !entry.enText) continue;
      const en = entry.enText;
      let result = "";
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
    if (e.ctrlKey && e.key === "f") {
      e.preventDefault();
      toolbarRef?.focusFilter();
      return;
    }
    if (e.ctrlKey && e.key === "h") {
      e.preventDefault();
      findReplaceVisible = !findReplaceVisible;
      return;
    }
    if (e.ctrlKey && e.key === "d") {
      e.preventDefault();
      dictVisible = !dictVisible;
      return;
    }
    if (e.ctrlKey && e.key === "g") {
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
  {#if error}
    <div class="error-bar">
      <span>{error}</span>
      <button class="error-dismiss btn-icon" onclick={() => (error = null)}>
        <XIcon size={14} />
      </button>
    </div>
  {/if}

  {#if !hasProject}
    <ProjectHome
      onNewProject={handleNewProject}
      onImportProject={handleImportProject}
      onOpenProject={handleOpenProject}
      {loading}
    />
  {:else if settingsVisible}
    <SettingsView
      projectName={projectName ?? ""}
      bind:settings={projectSettings}
      onBack={() => (settingsVisible = false)}
      onRename={handleRename}
      onSaveSettings={handleSaveSettings}
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
      {projectName}
      saveDisabled={!modified}
      bind:filterText
    />

    <div class="main-area">
      <DictionaryPanel
        query={dictQuery}
        querySeq={dictQuerySeq}
        bind:visible={dictVisible}
        bind:width={dictWidth}
      />

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

<GoToLineDialog
  bind:visible={goToLineVisible}
  maxLine={entries.length}
  onGo={handleGoToLine}
/>

<UnsavedChangesDialog
  bind:visible={unsavedDialogVisible}
  onSave={handleUnsavedSave}
  onDiscard={handleUnsavedDiscard}
/>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .error-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: var(--color-error-bg);
    color: var(--color-error-text);
    font-size: 13px;
    border-bottom: 1px solid var(--color-error-border);

    .error-dismiss {
      color: var(--color-error-text);
      font-size: 15px;
      padding: 0 4px;
      opacity: 0.7;

      &:hover {
        opacity: 1;
        color: var(--color-error-text);
      }
    }
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
