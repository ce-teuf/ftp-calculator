/**
 * usePyodide — singleton wrapper around the Pyodide Web Worker.
 *
 * Usage:
 *   const py = usePyodide();
 *   await py.run(code, { data: { ESTR: [...] } });
 */

export type PyodideStatus = 'idle' | 'loading' | 'ready' | 'running' | 'error';

export interface PyodideResult {
  ok: boolean;
  result: number[] | null;
  resultTenors: string[] | null;
  stdout: string;
  error?: string;
}

interface PendingCall {
  resolve: (r: PyodideResult) => void;
  reject: (e: Error) => void;
}

// ── Singleton state ───────────────────────────────────────────────────────────

let worker: Worker | null = null;
let statusCallbacks: ((s: PyodideStatus, text?: string) => void)[] = [];
let pendingCalls: Map<string, PendingCall> = new Map();
let _status: PyodideStatus = 'idle';
let callCounter = 0;

function setStatus(s: PyodideStatus, text?: string) {
  _status = s;
  for (const cb of statusCallbacks) cb(s, text);
}

function getWorker(): Worker {
  if (worker) return worker;

  setStatus('loading');
  worker = new Worker(
    new URL('./workers/pyodide.worker.js', import.meta.url),
    { type: 'module' }
  );

  worker.onmessage = ({ data: msg }) => {
    if (msg.type === 'ready') {
      setStatus('ready');
      return;
    }
    if (msg.type === 'status') {
      setStatus('loading', msg.text);
      return;
    }

    // Response to a run() call
    const pending = pendingCalls.get(msg.id);
    if (!pending) return;
    pendingCalls.delete(msg.id);
    setStatus('ready');
    pending.resolve({
      ok:            msg.ok,
      result:        msg.result ?? null,
      resultTenors:  msg.resultTenors ?? null,
      stdout:        msg.stdout ?? '',
      error:         msg.error,
    });
  };

  worker.onerror = (e) => {
    setStatus('error', e.message);
    for (const { reject } of pendingCalls.values()) {
      reject(new Error(e.message));
    }
    pendingCalls.clear();
    worker = null; // allow re-init on next call
  };

  return worker;
}

// ── Public API ────────────────────────────────────────────────────────────────

export function usePyodide() {
  // Ensure worker is started (pre-warms Pyodide)
  getWorker();

  function onStatus(cb: (s: PyodideStatus, text?: string) => void): () => void {
    statusCallbacks.push(cb);
    cb(_status); // fire immediately with current state
    return () => {
      statusCallbacks = statusCallbacks.filter(x => x !== cb);
    };
  }

  function getStatus(): PyodideStatus {
    return _status;
  }

  function run(
    code: string,
    context: Record<string, unknown> = {}
  ): Promise<PyodideResult> {
    const w = getWorker();
    const id = `call-${++callCounter}`;
    setStatus('running');

    return new Promise<PyodideResult>((resolve, reject) => {
      pendingCalls.set(id, { resolve, reject });
      w.postMessage({ id, type: 'run', code, context });
    });
  }

  return { run, onStatus, getStatus };
}
