export function gc() {
  // Multiple synchronous full-GC passes are needed because JSC's
  // FinalizationRegistry processing is deferred: a first pass collects
  // leaf objects (e.g. TreeCursor), and a subsequent pass collects
  // objects that were only kept alive by those leaves (e.g. Tree).
  Bun.gc(true);
  Bun.gc(true);
}
