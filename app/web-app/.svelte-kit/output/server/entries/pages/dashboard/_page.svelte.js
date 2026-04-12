import { s as spread_props, e as ensure_array_like, d as escape_html, c as attr, a9 as derived } from "../../../chunks/renderer.js";
import { e as executions } from "../../../chunks/client.js";
import "echarts";
import { L as Layout_dashboard } from "../../../chunks/layout-dashboard.js";
import { I as Icon } from "../../../chunks/Icon.js";
function Download($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    /**
     * @license @lucide/svelte v1.3.0 - ISC
     *
     * ISC License
     *
     * Copyright (c) 2026 Lucide Icons and Contributors
     *
     * Permission to use, copy, modify, and/or distribute this software for any
     * purpose with or without fee is hereby granted, provided that the above
     * copyright notice and this permission notice appear in all copies.
     *
     * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
     * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
     * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
     * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
     * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
     * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
     * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
     *
     * ---
     *
     * The following Lucide icons are derived from the Feather project:
     *
     * airplay, alert-circle, alert-octagon, alert-triangle, aperture, arrow-down-circle, arrow-down-left, arrow-down-right, arrow-down, arrow-left-circle, arrow-left, arrow-right-circle, arrow-right, arrow-up-circle, arrow-up-left, arrow-up-right, arrow-up, at-sign, calendar, cast, check, chevron-down, chevron-left, chevron-right, chevron-up, chevrons-down, chevrons-left, chevrons-right, chevrons-up, circle, clipboard, clock, code, columns, command, compass, corner-down-left, corner-down-right, corner-left-down, corner-left-up, corner-right-down, corner-right-up, corner-up-left, corner-up-right, crosshair, database, divide-circle, divide-square, dollar-sign, download, external-link, feather, frown, hash, headphones, help-circle, info, italic, key, layout, life-buoy, link-2, link, loader, lock, log-in, log-out, maximize, meh, minimize, minimize-2, minus-circle, minus-square, minus, monitor, moon, more-horizontal, more-vertical, move, music, navigation-2, navigation, octagon, pause-circle, percent, plus-circle, plus-square, plus, power, radio, rss, search, server, share, shopping-bag, sidebar, smartphone, smile, square, table-2, tablet, target, terminal, trash-2, trash, triangle, tv, type, upload, x-circle, x-octagon, x-square, x, zoom-in, zoom-out
     *
     * The MIT License (MIT) (for the icons listed above)
     *
     * Copyright (c) 2013-present Cole Bemis
     *
     * Permission is hereby granted, free of charge, to any person obtaining a copy
     * of this software and associated documentation files (the "Software"), to deal
     * in the Software without restriction, including without limitation the rights
     * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
     * copies of the Software, and to permit persons to whom the Software is
     * furnished to do so, subject to the following conditions:
     *
     * The above copyright notice and this permission notice shall be included in all
     * copies or substantial portions of the Software.
     *
     * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
     * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
     * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
     * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
     * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
     * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
     * SOFTWARE.
     *
     */
    let { $$slots, $$events, ...props } = $$props;
    const iconNode = [
      ["path", { "d": "M12 15V3" }],
      ["path", { "d": "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }],
      ["path", { "d": "m7 10 5 5 5-5" }]
    ];
    Icon($$renderer2, spread_props([
      { name: "download" },
      /**
       * @component @name Download
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTIgMTVWMyIgLz4KICA8cGF0aCBkPSJNMjEgMTV2NGEyIDIgMCAwIDEtMiAySDVhMiAyIDAgMCAxLTItMnYtNCIgLz4KICA8cGF0aCBkPSJtNyAxMCA1IDUgNS01IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/download
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      props,
      {
        iconNode,
        children: ($$renderer3) => {
          props.children?.($$renderer3);
          $$renderer3.push(`<!---->`);
        },
        $$slots: { default: true }
      }
    ]));
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let execList = [];
    let loading = true;
    let error = null;
    let primaryId = "";
    let compareId = "";
    let dateFrom = "";
    let dateTo = "";
    async function load() {
      loading = true;
      error = null;
      try {
        execList = (await executions.list()).filter((e) => e.status === "completed");
      } catch (e) {
        error = e.message;
      } finally {
        loading = false;
      }
    }
    load();
    const hasResults = derived(() => false);
    $$renderer2.push(`<div class="dash-page svelte-x1i5gj"><header class="dash-header card svelte-x1i5gj"><div class="header-left svelte-x1i5gj"><span class="header-icon svelte-x1i5gj">`);
    Layout_dashboard($$renderer2, { size: 18 });
    $$renderer2.push(`<!----></span> <span class="header-title svelte-x1i5gj">Dashboard</span></div> <div class="header-controls svelte-x1i5gj"><div class="ctrl-group svelte-x1i5gj"><label class="ctrl-label svelte-x1i5gj">Exécution `);
    $$renderer2.select(
      { class: "ctrl-select", value: primaryId, disabled: loading },
      ($$renderer3) => {
        $$renderer3.option({ value: "" }, ($$renderer4) => {
          $$renderer4.push(`— Sélectionner —`);
        });
        $$renderer3.push(`<!--[-->`);
        const each_array = ensure_array_like(execList);
        for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
          let ex = each_array[$$index];
          $$renderer3.option({ value: ex.id }, ($$renderer4) => {
            $$renderer4.push(`${escape_html(ex.study_name ?? ex.id)}${escape_html(ex.label ? ` · ${ex.label}` : "")}`);
          });
        }
        $$renderer3.push(`<!--]-->`);
      },
      "svelte-x1i5gj"
    );
    $$renderer2.push(`</label> <label class="ctrl-label svelte-x1i5gj">Comparaison `);
    $$renderer2.select(
      {
        class: "ctrl-select",
        value: compareId,
        disabled: loading || !primaryId
      },
      ($$renderer3) => {
        $$renderer3.option({ value: "" }, ($$renderer4) => {
          $$renderer4.push(`— Aucune —`);
        });
        $$renderer3.push(`<!--[-->`);
        const each_array_1 = ensure_array_like(execList.filter((e) => e.id !== primaryId));
        for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
          let ex = each_array_1[$$index_1];
          $$renderer3.option({ value: ex.id }, ($$renderer4) => {
            $$renderer4.push(`${escape_html(ex.study_name ?? ex.id)}${escape_html(ex.label ? ` · ${ex.label}` : "")}`);
          });
        }
        $$renderer3.push(`<!--]-->`);
      },
      "svelte-x1i5gj"
    );
    $$renderer2.push(`</label></div> <div class="ctrl-group svelte-x1i5gj"><label class="ctrl-label svelte-x1i5gj">Du <input class="ctrl-month svelte-x1i5gj" type="month"${attr("value", dateFrom)}/></label> <label class="ctrl-label svelte-x1i5gj">Au <input class="ctrl-month svelte-x1i5gj" type="month"${attr("value", dateTo)}/></label> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div> `);
    if (hasResults()) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<button class="btn-sm btn-export svelte-x1i5gj">`);
      Download($$renderer2, { size: 12 });
      $$renderer2.push(`<!----> CSV</button>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div></header> `);
    if (loading) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<p class="loading" style="padding:40px 32px">Chargement des exécutions…</p>`);
    } else if (error) {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="alert-error" style="margin:24px 32px">${escape_html(error)}</div>`);
    } else {
      $$renderer2.push("<!--[2-->");
      $$renderer2.push(`<div class="empty-state" style="margin:60px auto;max-width:380px">`);
      Layout_dashboard($$renderer2, { size: 36, style: "margin:0 auto 14px;opacity:.25" });
      $$renderer2.push(`<!----> <p>Sélectionnez une exécution terminée pour visualiser les résultats FTP.</p></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
