import { d as escape_html, e as ensure_array_like, b as attr_class, aa as stringify } from "../../../chunks/renderer.js";
import { e as executions, s as studies } from "../../../chunks/client.js";
import "echarts";
import { P as Play } from "../../../chunks/play.js";
import { T as Trash_2 } from "../../../chunks/trash-2.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let execList = [];
    let loading = true;
    let error = null;
    let selectedId = null;
    let allStudies = [];
    async function loadAll() {
      loading = true;
      error = null;
      try {
        const [el, sl] = await Promise.all([executions.list(), studies.list()]);
        execList = el;
        allStudies = sl;
      } catch (e) {
        error = e.message;
      } finally {
        loading = false;
      }
    }
    loadAll();
    function statusClass(s) {
      return s === "completed" ? "badge-active" : s === "error" ? "badge-error" : s === "running" ? "badge-pending" : "badge-draft";
    }
    function statusLabel(s) {
      return s === "completed" ? "Terminée" : s === "error" ? "Erreur" : s === "running" ? "En cours" : "En attente";
    }
    function fmtDuration(ms) {
      if (!ms) return "—";
      if (ms < 1e3) return `${ms} ms`;
      return `${(ms / 1e3).toFixed(1)} s`;
    }
    $$renderer2.push(`<div class="page svelte-i178sk"><aside class="left-panel card svelte-i178sk"><div class="panel-header svelte-i178sk"><h2 class="svelte-i178sk">Exécutions</h2> <button class="btn-primary">`);
    Play($$renderer2, { size: 12 });
    $$renderer2.push(`<!----> Lancer</button></div> `);
    if (loading) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<p class="loading" style="padding:16px">Chargement…</p>`);
    } else if (error) {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="alert-error" style="margin:12px">${escape_html(error)}</div>`);
    } else if (execList.length === 0) {
      $$renderer2.push("<!--[2-->");
      $$renderer2.push(`<div class="empty-state" style="margin:16px;padding:32px 16px"><p>Aucune exécution</p> <p>Lancez votre première simulation.</p></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="exec-list svelte-i178sk"><!--[-->`);
      const each_array = ensure_array_like(execList);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let ex = each_array[$$index];
        $$renderer2.push(`<div${attr_class("exec-item svelte-i178sk", void 0, { "exec-item--active": selectedId === ex.id })}><div class="exec-row1 svelte-i178sk"><span class="exec-study svelte-i178sk">${escape_html(ex.study_name ?? "—")}</span> <span${attr_class(`badge ${stringify(statusClass(ex.status))}`, "svelte-i178sk")}>${escape_html(statusLabel(ex.status))}</span></div> `);
        if (ex.label) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<div class="exec-label svelte-i178sk">${escape_html(ex.label)}</div>`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--> <div class="exec-row2 svelte-i178sk"><span class="exec-meta svelte-i178sk">${escape_html(new Date(ex.created_at).toLocaleDateString("fr-FR"))}</span> <span class="exec-meta svelte-i178sk">${escape_html(fmtDuration(ex.duration_ms))}</span></div> <div class="exec-actions svelte-i178sk"><button class="btn-sm btn-danger">`);
        Trash_2($$renderer2, { size: 11 });
        $$renderer2.push(`<!----></button></div></div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></aside> <main class="right-panel svelte-i178sk">`);
    {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="empty-state" style="margin:40px auto;max-width:400px">`);
      Play($$renderer2, { size: 32, style: "margin:0 auto 12px;opacity:.3" });
      $$renderer2.push(`<!----> <p>Sélectionnez une exécution ou lancez-en une nouvelle</p></div>`);
    }
    $$renderer2.push(`<!--]--></main></div> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
  });
}
export {
  _page as default
};
