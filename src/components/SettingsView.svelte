<script lang="ts">
  import type { ProjectSettings } from "../lib/types";
  import { ArrowLeftIcon, SaveIcon } from "@lucide/svelte";

  let {
    projectName,
    settings = $bindable({ autoConfirmOnEnter: false }),
    onBack,
    onRename,
    onSaveSettings,
  }: {
    projectName: string;
    settings?: ProjectSettings;
    onBack: () => void;
    onRename: (name: string) => void;
    onSaveSettings: (settings: ProjectSettings) => void;
  } = $props();

  let nameInput = $state("");
  let draft: ProjectSettings = $state({ autoConfirmOnEnter: false });
  let activeTab: "project" | "editor" = $state("project");

  $effect(() => {
    nameInput = projectName;
    draft = { ...settings };
  });

  let hasChanges = $derived(
    nameInput.trim() !== projectName || draft.autoConfirmOnEnter !== settings.autoConfirmOnEnter,
  );

  function save() {
    const trimmedName = nameInput.trim();
    if (trimmedName && trimmedName !== projectName) {
      onRename(trimmedName);
    }
    if (draft.autoConfirmOnEnter !== settings.autoConfirmOnEnter) {
      settings = { ...draft };
      onSaveSettings(draft);
    }
  }
</script>

<div class="settings-view">
  <div class="settings-header">
    <button class="back-btn" onclick={onBack}>
      <ArrowLeftIcon size={16} />
      <span>Back</span>
    </button>
    <div class="header-spacer"></div>
    <button class="btn-primary save-btn" onclick={save} disabled={!hasChanges}>
      <SaveIcon size={14} />
      <span>Save</span>
    </button>
  </div>

  <div class="settings-body">
    <h1>Settings</h1>
    <div class="tabs-layout">
    <div class="tabs">
      <button
        class="tab"
        class:tab-active={activeTab === "project"}
        onclick={() => (activeTab = "project")}
      >
        Project
      </button>
      <button
        class="tab"
        class:tab-active={activeTab === "editor"}
        onclick={() => (activeTab = "editor")}
      >
        Editor
      </button>
    </div>

    <div class="tab-content">
      {#if activeTab === "project"}
        <div class="field-row">
          <label class="field-label" for="project-name">Project Name</label>
          <input id="project-name" type="text" bind:value={nameInput} class="field-input" />
        </div>
      {:else}
        <label class="check-row">
          <input type="checkbox" bind:checked={draft.autoConfirmOnEnter} />
          <div>
            <span class="check-label">Auto-confirm on Enter</span>
            <span class="check-hint">
              Pressing Enter to move to the next line will also mark the current line as confirmed.
            </span>
          </div>
        </label>
      {/if}
    </div>
    </div>
  </div>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg);
  }

  .settings-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);

    button {
      font-weight: 600;
    }
  }

  .header-spacer {
    flex: 1;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    font-size: 13px;
  }

  .save-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 12px;
    font-size: 13px;
  }

  .settings-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 28px 32px 0;

    h1 {
      font-size: 20px;
      font-weight: 600;
      color: var(--color-text);
      margin-bottom: 20px;
    }
  }

  .tabs-layout {
    display: flex;
    gap: 0;
    flex: 1;
    min-height: 0;
  }

  .tabs {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 160px;
    flex-shrink: 0;
    padding: 8px 0;
    border-right: 1px solid var(--color-border);
  }

  .tab {
    padding: 10px 20px;
    font-size: 16px;
    background: none;
    border: none;
    border-left: 2px solid transparent;
    border-radius: 0;
    color: var(--color-text-muted);
    cursor: pointer;
    text-align: left;
    transition: color 0.15s, border-color 0.15s, background 0.15s;

    &:hover {
      color: var(--color-text);
      background: var(--color-surface);
    }
  }

  .tab-active {
    color: var(--color-accent);
    border-left-color: var(--color-accent);
    background: var(--color-surface);

    &:hover {
      color: var(--color-accent);
    }
  }

  .tab-content {
    flex: 1;
    padding: 20px 28px;
    overflow-y: auto;
  }

  .field-row {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 0;
  }

  .field-label {
    font-size: 14px;
    color: var(--color-text);
    white-space: nowrap;
    min-width: 110px;
  }

  .field-input {
    flex: 1;
    padding: 6px 10px;
    max-width: 320px;
  }

  .check-row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 0;
    cursor: pointer;
    user-select: none;

    input[type="checkbox"] {
      width: 16px;
      height: 16px;
      accent-color: var(--color-accent);
      cursor: pointer;
      flex-shrink: 0;
      margin-top: 1px;
    }
  }

  .check-label {
    display: block;
    font-size: 14px;
    color: var(--color-text);
  }

  .check-hint {
    display: block;
    font-size: 12px;
    color: var(--color-text-muted);
    margin-top: 4px;
    line-height: 1.4;
  }
</style>
