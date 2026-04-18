<script lang="ts">
  import { BookOpenIcon, FileIcon, FolderIcon, PencilIcon, SettingsIcon } from '@lucide/svelte';

  import type { AppSettings, ProjectFiles, ProjectSettings } from '../lib/types';

  import CopyButton from './ui/CopyButton.svelte';
  import PathText from './ui/PathText.svelte';
  import ViewHeader from './ui/ViewHeader.svelte';

  let {
    projectName,
    files,
    settings = $bindable({}),
    appSettings = $bindable(),
    minAutoSaveIntervalSecs,
    onBack,
    onSave,
  }: {
    projectName: string;
    files: ProjectFiles;
    settings?: ProjectSettings;
    appSettings: AppSettings;
    minAutoSaveIntervalSecs: number;
    onBack: () => void;
    onSave: (name: string, settings: ProjectSettings, appSettings: AppSettings) => void;
  } = $props();

  let nameInput = $state('');
  let draftApp: AppSettings = $state({ ...appSettings });
  let activeTab: string = $state('Project');

  $effect(() => {
    nameInput = projectName;
    draftApp = { ...appSettings };
  });

  let hasChanges = $derived(
    nameInput.trim() !== projectName ||
      draftApp.autoConfirmOnEnter !== appSettings.autoConfirmOnEnter ||
      draftApp.partialSearch !== appSettings.partialSearch ||
      draftApp.autoSaveIntervalSecs !== appSettings.autoSaveIntervalSecs,
  );

  function handleBack() {
    if (hasChanges) {
      const trimmedName = nameInput.trim() || projectName;
      if (draftApp.autoSaveIntervalSecs > 0 && draftApp.autoSaveIntervalSecs < minAutoSaveIntervalSecs) {
        draftApp.autoSaveIntervalSecs = minAutoSaveIntervalSecs;
      }
      appSettings = { ...draftApp };
      onSave(trimmedName, settings, draftApp);
    }
    onBack();
  }
</script>

<div class="settings-view">
  <ViewHeader onBack={handleBack}>
    <SettingsIcon size={14} />
    Settings / <strong>{activeTab}</strong>
  </ViewHeader>

  <div class="settings-body">
    <nav class="sidebar">
      <div class="tab-group">
        <div class="group-label">This project</div>
        <button class="tab" class:tab-active={activeTab === 'Project'} onclick={() => (activeTab = 'Project')}>
          <span class="tab-icon"><FolderIcon size={14} /></span>
          Project
        </button>
      </div>
      <div class="tab-group">
        <div class="group-label">Application</div>
        <button class="tab" class:tab-active={activeTab === 'Editor'} onclick={() => (activeTab = 'Editor')}>
          <span class="tab-icon"><PencilIcon size={14} /></span>
          Editor
        </button>
        <button class="tab" class:tab-active={activeTab === 'Dictionary'} onclick={() => (activeTab = 'Dictionary')}>
          <span class="tab-icon"><BookOpenIcon size={14} /></span>
          Dictionary
        </button>
      </div>
    </nav>

    <main class="content">
      {#if activeTab === 'Project'}
        <h1>Project</h1>
        <p>Settings specific to this project.</p>

        <section class="card">
          <h2>General</h2>
          <p>Shown in the project picker and window title.</p>
          <div class="field">
            <label class="field-label" for="project-name">Project name</label>
            <input id="project-name" type="text" class="field-input" bind:value={nameInput} />
          </div>
        </section>

        <section class="card">
          <h2>Source files</h2>
          <div class="field">
            <span class="field-label">Japanese</span>
            <div class="path-row">
              <FileIcon size={14} />
              <PathText path={files.jp} />
              <CopyButton text={files.jp} title="Copy path" />
            </div>
          </div>
          <div class="field">
            <span class="field-label">English</span>
            <div class="path-row">
              <FileIcon size={14} />
              <PathText path={files.en} />
              <CopyButton text={files.en} title="Copy path" />
            </div>
          </div>
        </section>
      {:else if activeTab === 'Editor'}
        <h1>Editor</h1>
        <p>How the translation grid behaves while you work.</p>

        <section class="card">
          <h2>Editing behavior</h2>
          <label class="check-row">
            <input type="checkbox" bind:checked={draftApp.autoConfirmOnEnter} />
            <span>
              Auto-confirm on Enter
              <small>Pressing Enter to move to the next line will also mark the current line as confirmed.</small>
            </span>
          </label>
        </section>

        <section class="card">
          <h2>Auto-save</h2>
          <p>
            How often to write a recovery file when there are unsaved changes. Minimum
            {minAutoSaveIntervalSecs}s, set to 0 to disable.
          </p>
          <div class="field field-inline">
            <label class="field-label" for="auto-save-interval">Interval</label>
            <input
              id="auto-save-interval"
              type="number"
              min="0"
              max="600"
              class="field-input field-input-narrow"
              bind:value={draftApp.autoSaveIntervalSecs}
            />
            <span class="text-muted text-sm">seconds</span>
          </div>
        </section>
      {:else}
        <h1>Dictionary</h1>
        <p>How JMdict and Wiktionary lookups match what you type.</p>

        <section class="card">
          <h2>Search</h2>
          <label class="check-row">
            <input type="checkbox" bind:checked={draftApp.partialSearch} />
            <span>
              Partial search
              <small>
                Always use prefix matching instead of prioritizing exact matches. Applies to JMdict and Wiktionary.
              </small>
            </span>
          </label>
        </section>
      {/if}
    </main>
  </div>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg);
  }

  .settings-body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .sidebar {
    width: 240px;
    flex-shrink: 0;
    padding: 18px 10px;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
  }

  .tab-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .group-label {
    padding: 4px 12px 8px;
    font-size: 10px;
    font-weight: 700;
    color: var(--color-text-disabled);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 12px;
    width: 100%;
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    font-weight: 600;
    color: var(--color-text-muted);
    text-align: left;

    &:hover:not(.tab-active) {
      background: var(--color-input-bg);
    }

    &.tab-active {
      background: var(--color-input-bg);
      color: var(--color-text);

      .tab-icon {
        background: var(--color-accent);
        color: #fff;
      }
    }
  }

  .tab-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-md);
    background: var(--color-input-bg);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 28px 36px 48px;

    > h1 {
      font-size: 22px;
      font-weight: 600;
      margin: 0;
    }

    > h1 + p {
      font-size: 13px;
      color: var(--color-text-muted);
      margin: 4px 0 24px;
    }
  }

  .card {
    padding: 20px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);

    & + & {
      margin-top: 16px;
    }

    h2 {
      font-size: 13px;
      font-weight: 600;
      margin-bottom: 14px;
    }

    h2 + p {
      font-size: 12px;
      color: var(--color-text-muted);
      margin: -12px 0 14px;
      line-height: 1.45;
    }
  }

  .field {
    & + .field {
      margin-top: 14px;
    }
  }

  .field-inline {
    display: flex;
    align-items: center;
    gap: 10px;

    .field-label {
      margin-bottom: 0;
      min-width: 80px;
    }
  }

  .field-label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 6px;
  }

  .field-input {
    width: 100%;
    padding: 9px 12px;
    font-size: 14px;
  }

  .field-input-narrow {
    width: auto;
    max-width: 110px;
    padding: 7px 10px;
  }

  .path-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .check-row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    cursor: pointer;
    user-select: none;

    input[type='checkbox'] {
      width: 16px;
      height: 16px;
      accent-color: var(--color-accent);
      cursor: pointer;
      flex-shrink: 0;
      margin-top: 1px;
    }

    > span {
      font-size: 14px;
      color: var(--color-text);
    }

    small {
      display: block;
      font-size: 12px;
      color: var(--color-text-muted);
      margin-top: 4px;
      line-height: 1.45;
    }
  }
</style>
