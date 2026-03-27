<script lang="ts">
  import type { RecentProject } from "../lib/types";
  import { getVersion } from "@tauri-apps/api/app";
  import { XIcon, Trash2Icon, TriangleAlertIcon } from "@lucide/svelte";
  import {
    listRecentProjects,
    listAllProjects,
    removeRecentProject,
    deleteProject,
    openFileDialog,
    importProjectDialog,
    previewImport,
  } from "../lib/ipc";
  import LoadingOverlay from "./ui/LoadingOverlay.svelte";
  import Dialog from "./ui/Dialog.svelte";

  let {onNewProject, onImportProject, onOpenProject, loading = false}: {
    onNewProject: (name: string, jpPath: string, enPath: string) => void;
    onImportProject: (sourcePath: string, name: string, jp: string, en: string) => void;
    onOpenProject: (id: string) => void;
    loading?: boolean;
  } = $props();

  let recentProjects: RecentProject[] = $state([]);
  let allProjects: RecentProject[] = $state([]);
  let showNewForm = $state(false);
  let importSourcePath = $state("");
  let deleteTarget: RecentProject | null = $state(null);
  let newName = $state("");
  let newJpPath = $state("");
  let newEnPath = $state("");
  let formError = $state("");
  let appVersion = $state("");

  let isImport = $derived(importSourcePath !== "");

  getVersion().then((v) => (appVersion = v));

  function refreshLists() {
    listRecentProjects().then((r) => (recentProjects = r));
    listAllProjects().then((r) => (allProjects = r));
  }

  $effect(() => {
    refreshLists();
  });

  async function handlePickJp() {
    const path = await openFileDialog();
    if (path) newJpPath = path;
  }

  async function handlePickEn() {
    const path = await openFileDialog();
    if (path) newEnPath = path;
  }

  function handleCreate() {
    formError = "";
    if (!newName.trim()) {
      formError = "Project name is required";
      return;
    }
    if (!newJpPath) {
      formError = "Japanese file is required";
      return;
    }
    if (!newEnPath) {
      formError = "English file is required";
      return;
    }
    if (isImport) {
      onImportProject(importSourcePath, newName.trim(), newJpPath, newEnPath);
    } else {
      onNewProject(newName.trim(), newJpPath, newEnPath);
    }
  }

  async function handleImport() {
    const path = await importProjectDialog();
    if (!path) return;
    try {
      const preview = await previewImport(path);
      importSourcePath = path;
      newName = preview.name;
      newJpPath = "";
      newEnPath = "";
      formError = "";
      showNewForm = true;
    } catch (e) {
      formError = `${e}`;
      showNewForm = true;
    }
  }

  async function handleRemoveRecent(e: MouseEvent, id: string) {
    e.stopPropagation();
    await removeRecentProject(id);
    refreshLists();
  }

  function handleDeleteProject(e: MouseEvent, proj: RecentProject) {
    e.stopPropagation();
    deleteTarget = proj;
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    await deleteProject(deleteTarget.id);
    deleteTarget = null;
    refreshLists();
  }
</script>

<div class="home">
  <LoadingOverlay visible={loading} />
  <div class="home-left">
    <div class="list-section">
      <h2 class="section-title">Recent Projects</h2>
      {#if recentProjects.length > 0}
        <div class="project-list">
          {#each recentProjects as proj}
            <div class="project-item">
              <button class="project-name text-ellipsis" onclick={() => onOpenProject(proj.id)}>
                {proj.name}
              </button>
              <button
                class="btn-icon item-action"
                title="Remove from recent"
                onclick={(e) => handleRemoveRecent(e, proj.id)}
              >
                <XIcon size={14} />
              </button>
              <button
                class="btn-icon item-action delete-action"
                title="Delete project"
                onclick={(e) => handleDeleteProject(e, proj)}
              >
                <Trash2Icon size={14} />
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <div class="no-projects">No recent projects</div>
      {/if}
    </div>

    <div class="list-section">
      <h2 class="section-title">All Projects</h2>
      {#if allProjects.length > 0}
        <div class="project-list">
          {#each allProjects as proj}
            <div class="project-item">
              <button class="project-name text-ellipsis" onclick={() => onOpenProject(proj.id)}>
                {proj.name}
              </button>
              <button
                class="btn-icon item-action delete-action"
                title="Delete project"
                onclick={(e) => handleDeleteProject(e, proj)}
              >
                <Trash2Icon size={14} />
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <div class="no-projects">No projects</div>
      {/if}
    </div>
  </div>

  <div class="home-right">
    {#if !showNewForm}
      <div class="home-actions">
        <button class="new-btn" onclick={() => { importSourcePath = ""; showNewForm = true; }}>
          New Project
        </button>
        <button class="import-btn" onclick={handleImport}>
          Import Project
        </button>
      </div>
    {:else}
      <div class="new-form">
        <h2>{isImport ? "Import Project" : "New Project"}</h2>
        <div class="form-field">
          <label>Project Name</label>
          <input
            type="text"
            bind:value={newName}
            placeholder="My Translation"
          />
        </div>
        <div class="form-field">
          <label>Japanese File</label>
          <div class="file-pick">
            <span class="file-path text-ellipsis" title={newJpPath}>
              {newJpPath || "No file selected"}
            </span>
            <button onclick={handlePickJp}>Browse</button>
          </div>
        </div>
        <div class="form-field">
          <label>English File</label>
          <div class="file-pick">
            <span class="file-path text-ellipsis" title={newEnPath}>
              {newEnPath || "No file selected"}
            </span>
            <button onclick={handlePickEn}>Browse</button>
          </div>
        </div>
        {#if formError}
          <div class="form-error">{formError}</div>
        {/if}
        <div class="form-actions">
          <button class="btn-primary" onclick={handleCreate}>Create</button>
          <button onclick={() => { showNewForm = false; importSourcePath = ""; }}>Cancel</button>
        </div>
      </div>
    {/if}
  </div>

  {#if appVersion}
    <span class="version">v{appVersion}</span>
  {/if}
</div>

{#if deleteTarget}
  {@const name = deleteTarget.name}
  <Dialog visible={true} title="Delete Project" onClose={() => (deleteTarget = null)}>
    <div class="delete-confirm">
      <TriangleAlertIcon size={24} class="warning-icon" />
      <p>Delete project "{name}"? This cannot be undone.</p>
    </div>

    {#snippet actions()}
      <button class="btn-danger" onclick={confirmDelete}>Delete</button>
      <button onclick={() => (deleteTarget = null)}>Cancel</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .home {
    display: flex;
    height: 100%;
    overflow: hidden;
    position: relative;
  }

  .version {
    position: absolute;
    bottom: 8px;
    right: 12px;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .home-left {
    width: 280px;
    flex-shrink: 0;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .list-section {
    padding: 16px 0 8px;
  }

  .list-section + .list-section {
    border-top: 1px solid var(--color-border);
  }

  .section-title {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    padding: 0 16px 8px;
  }

  .project-list {
    display: flex;
    flex-direction: column;
  }

  .project-item {
    display: flex;
    align-items: center;

    &:hover {
      background: var(--color-surface-alt);

      .item-action {
        opacity: 0.6;
      }
    }

    .project-name {
      flex: 1;
      padding: 10px 16px;
      background: none;
      border: none;
      border-radius: 0;
      color: var(--color-text);
      cursor: pointer;
      text-align: left;
      font-size: 14px;
      min-width: 0;
    }

    .item-action {
      flex-shrink: 0;
      align-self: stretch;
      display: flex;
      align-items: center;
      border-radius: 0;
      padding: 0 6px;
      opacity: 0;
      transition: opacity 0.1s;

      &:hover {
        opacity: 1 !important;
      }

      &.delete-action:hover {
        color: var(--color-danger-light);
      }
    }
  }

  .no-projects {
    padding: 16px;
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .home-right {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
  }

  .home-actions {
    display: flex;
    gap: 16px;
    align-items: center;
  }

  .new-btn, .import-btn {
    padding: 32px 48px;
    font-size: 22px;
    border-radius: 8px;
  }

  .new-form {
    width: 100%;
    max-width: 640px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 24px;

    h2 {
      font-size: 16px;
      margin-bottom: 16px;
      color: var(--color-text);
    }
  }

  .form-field {
    margin-bottom: 12px;

    label {
      display: block;
      font-size: 12px;
      color: var(--color-text-muted);
      margin-bottom: 4px;
    }

    input {
      width: 100%;
      padding: 8px 10px;
    }
  }

  .file-pick {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .file-path {
    flex: 1;
    padding: 8px 10px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    font-size: 13px;
    color: var(--color-text-muted);
    direction: rtl;
    text-align: left;
  }

  .form-error {
    color: var(--color-danger-light);
    font-size: 13px;
    margin-bottom: 8px;
  }

  .form-actions {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }

  .delete-confirm {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    font-size: 14px;
    color: var(--color-text);

    :global(.warning-icon) {
      flex-shrink: 0;
      color: var(--color-warning);
    }

    p {
      line-height: 1.5;
    }
  }

  .btn-danger {
    background: var(--color-danger);
    border-color: var(--color-danger);
    color: #fff;

    &:hover:not(:disabled) {
      filter: brightness(1.15);
      background: var(--color-danger);
    }
  }
</style>
