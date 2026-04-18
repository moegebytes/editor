<script lang="ts" module>
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');

  function truncatePath(path: string, maxChars: number): string {
    if (path.length <= maxChars) return path;
    const ellipsis = '…';
    const available = maxChars - ellipsis.length;
    if (available <= 0) return ellipsis;
    const truncated = path.slice(-available);
    const sepIndex = truncated.search(/[/\\]/);
    if (sepIndex > 0 && sepIndex < truncated.length - 1) {
      return ellipsis + truncated.slice(sepIndex);
    }
    return ellipsis + truncated;
  }
</script>

<script lang="ts">
  let { path, placeholder }: { path: string; placeholder?: string } = $props();

  let containerEl: HTMLSpanElement | undefined = $state();
  let containerWidth = $state(0);
  let charWidth = $state(0);

  function measureCharWidth(el: HTMLSpanElement): number {
    if (!ctx) return 8;
    const style = getComputedStyle(el);
    ctx.font = `${style.fontSize} ${style.fontFamily}`;
    return ctx.measureText('M').width;
  }

  let displayPath = $derived.by(() => {
    if (!path) return placeholder ?? '';
    if (containerWidth <= 0 || charWidth <= 0) return path;

    const maxChars = Math.floor(containerWidth / charWidth);
    if (path.length <= maxChars) return path;

    return truncatePath(path, maxChars);
  });

  $effect(() => {
    if (!containerEl) return;

    charWidth = measureCharWidth(containerEl);

    const observer = new ResizeObserver((entries) => {
      containerWidth = entries[0].contentRect.width;
    });
    observer.observe(containerEl);
    containerWidth = containerEl.clientWidth;

    return () => observer.disconnect();
  });
</script>

<span class="path-text" bind:this={containerEl} title={path}>
  {displayPath}
</span>

<style>
  .path-text {
    flex: 1;
    min-width: 0;
    display: block;
    overflow: hidden;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: 12px;
    user-select: text;
  }
</style>
