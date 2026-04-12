import { d as escape_html, e as ensure_array_like, b as attr_class } from "../../../chunks/renderer.js";
import { b as studyUnits, h as hypercubes, p as portfolios, a as amortSchedules } from "../../../chunks/client.js";
import "../../../chunks/MonthPicker.svelte_svelte_type_style_lang.js";
import { P as Plus } from "../../../chunks/plus.js";
import { P as Pencil } from "../../../chunks/pencil.js";
import { T as Trash_2 } from "../../../chunks/trash-2.js";
import { F as Flask_conical } from "../../../chunks/flask-conical.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let unitList = [];
    let loading = true;
    let error = null;
    let selectedId = null;
    let allHypercubes = [];
    let allPortfolios = [];
    let allSchedules = [];
    async function loadAll() {
      loading = true;
      error = null;
      try {
        const [units, hcs, ports, scheds] = await Promise.all([
          studyUnits.list(),
          hypercubes.list(),
          portfolios.list(),
          amortSchedules.list()
        ]);
        unitList = units;
        allHypercubes = hcs;
        allPortfolios = ports;
        allSchedules = scheds;
      } catch (e) {
        error = e.message;
      } finally {
        loading = false;
      }
    }
    loadAll();
    let $$settled = true;
    let $$inner_renderer;
    function $$render_inner($$renderer3) {
      $$renderer3.push(`<div class="page svelte-1x9gcqo"><aside class="left-panel card svelte-1x9gcqo"><div class="panel-header svelte-1x9gcqo"><h2 class="svelte-1x9gcqo">Study Units</h2> <button class="btn-primary">`);
      Plus($$renderer3, { size: 13 });
      $$renderer3.push(`<!----> Nouvelle</button></div> `);
      if (loading) {
        $$renderer3.push("<!--[0-->");
        $$renderer3.push(`<p class="loading" style="padding:16px">Chargement…</p>`);
      } else if (error) {
        $$renderer3.push("<!--[1-->");
        $$renderer3.push(`<div class="alert-error" style="margin:12px">${escape_html(error)}</div>`);
      } else if (unitList.length === 0) {
        $$renderer3.push("<!--[2-->");
        $$renderer3.push(`<div class="empty-state" style="margin:16px;padding:32px 16px"><p>Aucune study unit</p> <p>Créez-en une pour lier un hypercube à un portfolio.</p></div>`);
      } else {
        $$renderer3.push("<!--[-1-->");
        $$renderer3.push(`<div class="unit-list svelte-1x9gcqo"><!--[-->`);
        const each_array = ensure_array_like(unitList);
        for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
          let unit = each_array[$$index];
          $$renderer3.push(`<div${attr_class("unit-item svelte-1x9gcqo", void 0, { "unit-item--active": selectedId === unit.id })}><div class="unit-row1 svelte-1x9gcqo"><span class="unit-name svelte-1x9gcqo">${escape_html(unit.name)}</span> `);
          if (unit.is_valid) {
            $$renderer3.push("<!--[0-->");
            $$renderer3.push(`<span class="badge badge-active">✓ Valide</span>`);
          } else {
            $$renderer3.push("<!--[-1-->");
            $$renderer3.push(`<span class="badge badge-draft">Non validée</span>`);
          }
          $$renderer3.push(`<!--]--></div> <div class="unit-row2 svelte-1x9gcqo"><span class="unit-meta-item svelte-1x9gcqo">HC: ${escape_html(unit.hypercube_name)}</span></div> <div class="unit-row2 svelte-1x9gcqo"><span class="unit-meta-item svelte-1x9gcqo">P: ${escape_html(unit.portfolio_name)}</span> <span class="unit-count svelte-1x9gcqo">${escape_html(unit.assignment_count)} assign.</span></div> <div class="unit-actions svelte-1x9gcqo"><button class="btn-sm" title="Modifier">`);
          Pencil($$renderer3, { size: 11 });
          $$renderer3.push(`<!----></button> <button class="btn-sm btn-danger" title="Supprimer">`);
          Trash_2($$renderer3, { size: 11 });
          $$renderer3.push(`<!----></button></div></div>`);
        }
        $$renderer3.push(`<!--]--></div>`);
      }
      $$renderer3.push(`<!--]--></aside> <main class="right-panel svelte-1x9gcqo">`);
      {
        $$renderer3.push("<!--[0-->");
        $$renderer3.push(`<div class="empty-state" style="margin:40px auto;max-width:400px">`);
        Flask_conical($$renderer3, { size: 32, style: "margin:0 auto 12px;opacity:.3" });
        $$renderer3.push(`<!----> <p>Sélectionnez une study unit pour voir son détail</p></div>`);
      }
      $$renderer3.push(`<!--]--></main></div> `);
      {
        $$renderer3.push("<!--[-1-->");
      }
      $$renderer3.push(`<!--]--> `);
      {
        $$renderer3.push("<!--[-1-->");
      }
      $$renderer3.push(`<!--]-->`);
    }
    do {
      $$settled = true;
      $$inner_renderer = $$renderer2.copy();
      $$render_inner($$inner_renderer);
    } while (!$$settled);
    $$renderer2.subsume($$inner_renderer);
  });
}
export {
  _page as default
};
