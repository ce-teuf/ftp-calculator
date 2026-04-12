import { d as escape_html, e as ensure_array_like, b as attr_class, aa as stringify } from "../../../chunks/renderer.js";
import { s as studies, b as studyUnits } from "../../../chunks/client.js";
import { P as Plus } from "../../../chunks/plus.js";
import { B as Book_open } from "../../../chunks/book-open.js";
import { P as Pencil } from "../../../chunks/pencil.js";
import { T as Trash_2 } from "../../../chunks/trash-2.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let studyList = [];
    let loading = true;
    let error = null;
    let selectedId = null;
    let allUnits = [];
    async function loadAll() {
      loading = true;
      error = null;
      try {
        const [sl, ul] = await Promise.all([studies.list(), studyUnits.list()]);
        studyList = sl;
        allUnits = ul;
      } catch (e) {
        error = e.message;
      } finally {
        loading = false;
      }
    }
    loadAll();
    function statusLabel(s) {
      return s === "draft" ? "Brouillon" : s === "ready" ? "Prête" : "Archivée";
    }
    function statusClass(s) {
      return s === "ready" ? "badge-active" : s === "archived" ? "badge-archived" : "badge-draft";
    }
    $$renderer2.push(`<div class="page svelte-qoay0y"><aside class="left-panel card svelte-qoay0y"><div class="panel-header svelte-qoay0y"><h2 class="svelte-qoay0y">Studies</h2> <button class="btn-primary">`);
    Plus($$renderer2, { size: 13 });
    $$renderer2.push(`<!----> Nouvelle</button></div> `);
    if (loading) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<p class="loading" style="padding:16px">Chargement…</p>`);
    } else if (error) {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="alert-error" style="margin:12px">${escape_html(error)}</div>`);
    } else if (studyList.length === 0) {
      $$renderer2.push("<!--[2-->");
      $$renderer2.push(`<div class="empty-state" style="margin:16px;padding:32px 16px"><p>Aucune étude</p> <p>Créez une étude pour regrouper des study units.</p></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="study-list svelte-qoay0y"><!--[-->`);
      const each_array = ensure_array_like(studyList);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let s = each_array[$$index];
        $$renderer2.push(`<div${attr_class("study-item svelte-qoay0y", void 0, { "study-item--active": selectedId === s.id })}><div class="study-row1 svelte-qoay0y"><span class="study-name svelte-qoay0y">${escape_html(s.name)}</span> <span${attr_class(`badge ${stringify(statusClass(s.status))}`, "svelte-qoay0y")}>${escape_html(statusLabel(s.status))}</span></div> <div class="study-row2 svelte-qoay0y">`);
        if (s.unit_count > 0) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<span class="study-meta svelte-qoay0y">${escape_html(s.valid_unit_count)}/${escape_html(s.unit_count)} unité(s) valide(s)</span>`);
        } else {
          $$renderer2.push("<!--[-1-->");
          $$renderer2.push(`<span class="study-meta empty-meta svelte-qoay0y">Aucune study unit</span>`);
        }
        $$renderer2.push(`<!--]--></div> <div class="study-actions svelte-qoay0y"><button class="btn-sm" title="Modifier">`);
        Pencil($$renderer2, { size: 11 });
        $$renderer2.push(`<!----></button> <button class="btn-sm btn-danger" title="Supprimer">`);
        Trash_2($$renderer2, { size: 11 });
        $$renderer2.push(`<!----></button></div></div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></aside> <main class="right-panel svelte-qoay0y">`);
    {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="empty-state" style="margin:40px auto;max-width:400px">`);
      Book_open($$renderer2, { size: 32, style: "margin:0 auto 12px;opacity:.3" });
      $$renderer2.push(`<!----> <p>Sélectionnez une étude pour voir son contenu</p></div>`);
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
