<script lang="ts">
  import Dialog from './ui/Dialog.svelte';
  import { ChevronDownIcon, ChevronRightIcon } from '@lucide/svelte';
  import { getEnvironmentInfo } from '../lib/ipc';
  import type { AppSettings, EnvironmentInfo, ProjectSettings } from '../lib/types';

  let {
    visible = $bindable(false),
    projectName = null,
    stats = null,
    appSettings,
    projectSettings = null,
  }: {
    visible?: boolean;
    projectName?: string | null;
    stats?: { totalText: number; translated: number; confirmed: number } | null;
    appSettings: AppSettings;
    projectSettings?: ProjectSettings | null;
  } = $props();

  let detailsOpen = $state(false);
  let envInfo: EnvironmentInfo | null = $state(null);
  let envError: string | null = $state(null);

  $effect(() => {
    if (visible && !envInfo) {
      getEnvironmentInfo()
        .then((info) => (envInfo = info))
        .catch((e) => (envError = String(e)));
    }
  });

  let detailsText = $derived.by(() => {
    if (!envInfo) return envError ?? 'Loading...';
    const lines: string[] = [
      `App: ${envInfo.appName} ${envInfo.appVersion} (${envInfo.debug ? 'debug' : 'release'})`,
      `Tauri: ${envInfo.tauriVersion}`,
      `WebView: ${envInfo.webviewVersion}`,
      `Platform: ${envInfo.os}`,
    ];
    lines.push(`Settings: ${JSON.stringify(appSettings)}`);
    if (projectName) {
      lines.push('');
      lines.push(`Project: ${projectName}`);
      if (stats) {
        lines.push(`Entries: ${stats.totalText} total, ${stats.translated} translated, ${stats.confirmed} confirmed`);
      }
      if (projectSettings) {
        lines.push(`Settings: ${JSON.stringify(projectSettings)}`);
      }
    }
    return lines.join('\n');
  });

  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    return () => clearTimeout(copyTimer);
  });

  function handleCopy() {
    navigator.clipboard.writeText(detailsText);
    copied = true;
    clearTimeout(copyTimer);
    copyTimer = setTimeout(() => (copied = false), 2000);
  }
</script>

<Dialog title="About" bind:visible>
  <div class="about-content">
    <h3 class="app-title">{envInfo?.appName ?? 'Yona'}</h3>
    <p class="app-version">Version {envInfo?.appVersion ?? '...'}</p>
    <p class="app-description">Desktop editor for translating Visual Novel script files.</p>

    <div class="credits">
      <h4>Credits</h4>
      <ul>
        <li>
          <strong>JMdict</strong> — Japanese-English dictionary
          <span class="credit-note">Electronic Dictionary Research and Development Group</span>
        </li>
        <li>
          <strong>KANJIDIC2</strong> — Kanji dictionary
          <span class="credit-note">Electronic Dictionary Research and Development Group</span>
        </li>
        <li>
          <strong>Wiktionary</strong> — multilingual dictionary data
          <span class="credit-note">Wikimedia Foundation, via Wiktextract project</span>
        </li>
        <li>
          <strong>vibrato</strong> — Japanese morphological analysis
          <span class="credit-note">with IPADIC (MeCab 2.7.0)</span>
        </li>
      </ul>
    </div>

    <div class="details-section">
      <button class="details-toggle" onclick={() => (detailsOpen = !detailsOpen)}>
        {#if detailsOpen}
          <ChevronDownIcon size={14} />
        {:else}
          <ChevronRightIcon size={14} />
        {/if}
        Environment details
      </button>
      {#if detailsOpen}
        <div class="details-body">
          <pre>{detailsText}</pre>
          <button class="copy-btn" class:copied onclick={handleCopy}>
            {copied ? 'Copied!' : 'Copy'}
          </button>
        </div>
      {/if}
    </div>
  </div>
</Dialog>

<style>
  .about-content {
    min-width: 360px;
  }

  .app-title {
    font-size: 18px;
    font-weight: 700;
    margin: 0;
  }

  .app-version {
    font-size: 13px;
    color: var(--color-text-muted);
    margin: 2px 0 12px;
  }

  .app-description {
    font-size: 13px;
    margin: 0 0 16px;
  }

  .credits {
    margin-bottom: 16px;

    h4 {
      font-size: 13px;
      font-weight: 600;
      margin: 0 0 6px;
      color: var(--color-text-muted);
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    ul {
      list-style: none;
      padding: 0;
      margin: 0;
      display: flex;
      flex-direction: column;
      gap: 4px;
    }

    li {
      font-size: 13px;
      line-height: 1.4;
    }

    .credit-note {
      color: var(--color-text-muted);
      font-size: 12px;
      display: block;
      padding-left: 4px;
    }
  }

  .details-section {
    border-top: 1px solid var(--color-border);
    padding-top: 12px;
  }

  .details-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-muted);
    background: none;
    border: none;
    padding: 2px 0;
    cursor: pointer;

    &:hover {
      color: var(--color-text);
    }
  }

  .details-body {
    margin-top: 8px;
    position: relative;
  }

  .details-body pre {
    font-size: 12px;
    line-height: 1.5;
    background: var(--color-surface-alt);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    padding: 10px 12px;
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
  }

  .copy-btn {
    position: absolute;
    top: 6px;
    right: 6px;
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 3px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    cursor: pointer;

    &:hover {
      color: var(--color-text);
      border-color: var(--color-text-muted);
    }

    &.copied {
      color: var(--color-success-text);
      border-color: var(--color-success-border);
    }
  }
</style>
