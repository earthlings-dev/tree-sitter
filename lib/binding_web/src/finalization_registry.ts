/**
 * Callback invoked each time a registered object is finalized by the GC.
 * Used by tests to observe automatic memory reclamation without
 * monkey‐patching `globalThis.FinalizationRegistry`.
 */
export type FinalizeObserver = () => void;

let observer: FinalizeObserver | null = null;

/**
 * Subscribe to finalization events.  The observer is called once for every
 * object that is reclaimed via a {@link Finalizer}.  Returns a teardown
 * function that removes the subscription.
 *
 * @internal  – intended for test harnesses only.
 */
export function onFinalize(fn: FinalizeObserver): () => void {
  observer = fn;
  return () => { observer = null; };
}

/**
 * Create a {@link FinalizationRegistry} that (a) calls `handler` to free
 * native resources and (b) notifies the current {@link onFinalize} observer,
 * if any.
 *
 * The registry is created eagerly so it inherits whatever
 * `FinalizationRegistry` constructor is on the global at module‐load time.
 * Observability is decoupled: the observer is checked at callback‐invocation
 * time, so tests can subscribe/unsubscribe independently of module load order.
 */
export function newFinalizer<T>(handler: (value: T) => void): FinalizationRegistry<T> | undefined {
  try {
    return new FinalizationRegistry<T>((value) => {
      handler(value);
      observer?.();
    });
  } catch (e) {
    console.error('Unsupported FinalizationRegistry:', e);
    return;
  }
}
