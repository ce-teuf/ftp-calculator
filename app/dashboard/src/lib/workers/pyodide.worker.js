/**
 * Pyodide Web Worker — runs CPython (WASM) off the main thread.
 *
 * Message protocol (in):
 *   { id: string, type: 'run', code: string, context: Record<string, any> }
 *   { id: string, type: 'init' }   (pre-warm on startup)
 *
 * Message protocol (out):
 *   { id: string, ok: true,  result: any,    stdout: string }
 *   { id: string, ok: false, error: string,  stdout: string }
 *   { id: string, type: 'ready' }  (when packages are loaded)
 *   { id: string, type: 'status', text: string }  (progress messages)
 */

// Load from CDN — pinned version for reproducibility
const PYODIDE_URL = 'https://cdn.jsdelivr.net/pyodide/v0.27.3/full/pyodide.mjs';

let pyodide = null;
let initPromise = null;
let stdoutLines = [];

function sendStatus(text) {
  self.postMessage({ type: 'status', text });
}

async function initPyodide() {
  sendStatus('Chargement de Pyodide…');
  const { loadPyodide } = await import(PYODIDE_URL);

  pyodide = await loadPyodide({
    stdout: (text) => { stdoutLines.push(text); },
    stderr: (text) => { stdoutLines.push('[stderr] ' + text); },
  });

  sendStatus('Chargement de numpy, pandas, scipy…');
  await pyodide.loadPackage(['numpy', 'pandas', 'scipy']);

  sendStatus('Prêt');
  self.postMessage({ type: 'ready' });
}

function getInitPromise() {
  if (!initPromise) {
    initPromise = initPyodide().catch(err => {
      initPromise = null; // allow retry
      throw err;
    });
  }
  return initPromise;
}

// Pre-warm immediately when worker starts
getInitPromise();

self.onmessage = async ({ data: msg }) => {
  if (msg.type === 'init') return; // pre-warm already started above

  const { id, code, context } = msg;
  stdoutLines = [];

  try {
    await getInitPromise();

    // Inject context variables into Python globals
    // context = { data: { "ESTR": [{date, tenor, rate}, ...], ... } }
    pyodide.globals.set('_js_context', pyodide.toPy(context));

    // Bootstrap: expose context vars at top level, redirect stdout capture
    const bootstrap = `
import sys, io as _io
_stdout_capture = _io.StringIO()
sys.stdout = _stdout_capture

# Expose context dict as top-level variables
_ctx = _js_context.to_py()
data = _ctx.get('data', {})

# Reset result
result = None
result_tenors = None
`;

    await pyodide.runPythonAsync(bootstrap);
    await pyodide.runPythonAsync(code);

    // Capture stdout
    const stdout = pyodide.runPython('_stdout_capture.getvalue()');

    // Retrieve result
    const rawResult = pyodide.globals.get('result');
    const rawTenors = pyodide.globals.get('result_tenors');

    let result = null;
    if (rawResult !== null && rawResult !== undefined) {
      result = rawResult?.toJs ? rawResult.toJs({ dict_converter: Object.fromEntries }) : rawResult;
    }

    let resultTenors = null;
    if (rawTenors !== null && rawTenors !== undefined) {
      resultTenors = rawTenors?.toJs ? rawTenors.toJs() : rawTenors;
    }

    self.postMessage({
      id,
      ok: true,
      result,
      resultTenors,
      stdout: stdout + stdoutLines.join('\n'),
    });

  } catch (err) {
    self.postMessage({
      id,
      ok: false,
      error: err.message ?? String(err),
      stdout: stdoutLines.join('\n'),
    });
  }
};
