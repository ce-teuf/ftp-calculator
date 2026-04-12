import { d as escape_html, e as ensure_array_like, b as attr_class } from "../../../chunks/renderer.js";
import { p as portfolios, o as outstandingVectors, a as amortSchedules } from "../../../chunks/client.js";
import "echarts";
import { P as Plus } from "../../../chunks/plus.js";
import { B as Briefcase } from "../../../chunks/briefcase.js";
import { P as Pencil } from "../../../chunks/pencil.js";
import { T as Trash_2 } from "../../../chunks/trash-2.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let portfolioList = [];
    let loading = true;
    let error = null;
    let selectedId = null;
    async function loadAll() {
      loading = true;
      error = null;
      const [pRes, vRes, sRes] = await Promise.allSettled([
        portfolios.list(),
        outstandingVectors.list(),
        amortSchedules.list()
      ]);
      if (pRes.status === "fulfilled") portfolioList = pRes.value;
      else error = pRes.reason?.message ?? "Erreur chargement portfolios";
      if (vRes.status === "fulfilled") vRes.value;
      if (sRes.status === "fulfilled") sRes.value;
      loading = false;
    }
    loadAll();
    $$renderer2.push(`<div class="tab-content"><div class="tab-header"><h2>Portfolios</h2> <button class="btn-primary">`);
    Plus($$renderer2, { size: 15 });
    $$renderer2.push(`<!----> Nouveau portfolio</button></div> `);
    if (error) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="alert-error">${escape_html(error)}</div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="layout svelte-1srrmo2"><div class="list-panel card svelte-1srrmo2">`);
    if (loading) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<p class="loading" style="padding:20px 16px">Chargement…</p>`);
    } else if (portfolioList.length === 0 && true) {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="empty-state"><p>Aucun portfolio créé.</p> <button class="btn-primary">`);
      Plus($$renderer2, { size: 14 });
      $$renderer2.push(`<!----> Créer le premier</button></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<!--[-->`);
      const each_array = ensure_array_like(portfolioList);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let p = each_array[$$index];
        $$renderer2.push(`<div${attr_class("list-item svelte-1srrmo2", void 0, {
          "list-item--active": selectedId === p.id && true
        })} role="button" tabindex="0"><div class="list-item-header svelte-1srrmo2"><span class="list-item-name svelte-1srrmo2">${escape_html(p.name)}</span></div> <div class="list-item-stats svelte-1srrmo2"><span>${escape_html(p.vector_count)} vecteur${escape_html(p.vector_count !== 1 ? "s" : "")}</span> <span class="dot svelte-1srrmo2">·</span> <span>${escape_html(p.schedule_count)} schedule${escape_html(p.schedule_count !== 1 ? "s" : "")}</span> <span class="dot svelte-1srrmo2">·</span> <span>${escape_html(p.pair_count)} paire${escape_html(p.pair_count !== 1 ? "s" : "")}</span></div> `);
        if (p.description) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<div class="list-item-desc svelte-1srrmo2">${escape_html(p.description)}</div>`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--> <div class="list-item-actions svelte-1srrmo2"><button class="btn-sm">`);
        Pencil($$renderer2, { size: 12 });
        $$renderer2.push(`<!----></button> <button class="btn-sm btn-danger">`);
        Trash_2($$renderer2, { size: 12 });
        $$renderer2.push(`<!----></button></div></div>`);
      }
      $$renderer2.push(`<!--]-->`);
    }
    $$renderer2.push(`<!--]--></div> <div class="detail-panel svelte-1srrmo2">`);
    {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="card empty-detail svelte-1srrmo2">`);
      Briefcase($$renderer2, { size: 40, color: "#d1d5db" });
      $$renderer2.push(`<!----> <p>Sélectionnez un portfolio<br/>ou créez-en un nouveau.</p></div>`);
    }
    $$renderer2.push(`<!--]--></div></div></div>`);
  });
}
export {
  _page as default
};
