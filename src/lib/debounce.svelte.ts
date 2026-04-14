export function useDebouncedValue<T>(source: () => T, ms: number): { readonly value: T } {
  let value = $state(source());
  let timer: ReturnType<typeof setTimeout>;

  $effect(() => {
    const next = source();
    clearTimeout(timer);
    timer = setTimeout(() => (value = next), ms);
    return () => clearTimeout(timer);
  });

  return {
    get value() {
      return value;
    },
  };
}
