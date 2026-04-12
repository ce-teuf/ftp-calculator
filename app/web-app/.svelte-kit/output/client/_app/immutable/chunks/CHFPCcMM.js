import{e as We,i as ze,g as Fe,b as Ue,d as Ve,j as Be,n as qe,k as Ye,a as de,l as Ge,c as Ke}from"./Cjriaz1E.js";import{U as K,b as ie,aM as Te,h as S,c as H,av as Ne,a as re,v as y,r as je,af as Xe,s as ve,d as P,e as D,aD as Je,aK as Qe,ao as he,J as j,aN as O,V as q,aO as Ze,X as $e,a7 as xe,aP as er,aQ as rr,L as ar,ad as ge,aR as ir,a2 as fr,Q as ke,T as we,aS as $,ac as Ce,aT as nr,aU as tr,aH as sr,W as ur,R as X,aJ as fe,aV as Ie,aE as lr,E as Oe,aW as or,au as cr,aX as dr,o as vr,O as Le,aY as Me,k as ne,I as hr,aZ as gr,a_ as _r,a$ as br,b0 as pr,b1 as Z,b2 as Ar,b3 as Er,b4 as Sr,b5 as Tr,b6 as Nr,b7 as kr,b8 as wr,b9 as Cr,p as Ir,f as Or,y as Lr,g as Mr,z as Rr,n as mr,B as yr,F as _e,K as Dr}from"./DMwxfA9P.js";import{B as Re,p as W,r as Pr}from"./By017JCv.js";function Hr(e,r){return r}function Wr(e,r,a){for(var i=[],f=r.length,n,t=r.length,s=0;s<f;s++){let g=r[s];we(g,()=>{if(n){if(n.pending.delete(g),n.done.add(g),n.pending.size===0){var c=e.outrogroups;ae(e,fe(n.done)),c.delete(n),c.size===0&&(e.outrogroups=null)}}else t-=1},!1)}if(t===0){var o=i.length===0&&a!==null;if(o){var d=a,l=d.parentNode;sr(l),l.append(d),e.items.clear()}ae(e,r,!o)}else n={pending:new Set(r),done:new Set},(e.outrogroups??(e.outrogroups=new Set)).add(n)}function ae(e,r,a=!0){var i;if(e.pending.size>0){i=new Set;for(const t of e.pending.values())for(const s of t)i.add(e.items.get(s).e)}for(var f=0;f<r.length;f++){var n=r[f];if(i!=null&&i.has(n)){n.f|=O;const t=document.createDocumentFragment();ur(n,t)}else X(r[f],a)}}var be;function zr(e,r,a,i,f,n=null){var t=e,s=new Map,o=(r&Te)!==0;if(o){var d=e;t=S?H(Ne(d)):d.appendChild(K())}S&&re();var l=null,g=xe(()=>{var E=a();return Ie(E)?E:E==null?[]:fe(E)}),c,p=new Map,A=!0;function k(E){(C.effect.f&fr)===0&&(C.pending.delete(E),C.fallback=l,Fr(C,c,t,r,i),l!==null&&(c.length===0?(l.f&O)===0?ke(l):(l.f^=O,V(l,null,t)):we(l,()=>{l=null})))}function u(E){C.pending.delete(E)}var v=ie(()=>{c=y(g);var E=c.length;let b=!1;if(S){var R=je(t)===Xe;R!==(E===0)&&(t=ve(),H(t),P(!1),b=!0)}for(var _=new Set,h=j,N=$e(),w=0;w<E;w+=1){S&&D.nodeType===Je&&D.data===Qe&&(t=D,b=!0,P(!1));var T=c[w],L=i(T,w),I=A?null:s.get(L);I?(I.v&&he(I.v,T),I.i&&he(I.i,w),N&&h.unskip_effect(I.e)):(I=Ur(s,A?t:be??(be=K()),T,L,w,f,r,a),A||(I.e.f|=O),s.set(L,I)),_.add(L)}if(E===0&&n&&!l&&(A?l=q(()=>n(t)):(l=q(()=>n(be??(be=K()))),l.f|=O)),E>_.size&&Ze(),S&&E>0&&H(ve()),!A)if(p.set(h,_),N){for(const[Y,G]of s)_.has(Y)||h.skip_effect(G.e);h.oncommit(k),h.ondiscard(u)}else k(h);b&&P(!0),y(g)}),C={effect:v,items:s,pending:p,outrogroups:null,fallback:l};A=!1,S&&(t=D)}function z(e){for(;e!==null&&(e.f&nr)===0;)e=e.next;return e}function Fr(e,r,a,i,f){var T,L,I,Y,G,se,ue,le,oe;var n=(i&tr)!==0,t=r.length,s=e.items,o=z(e.effect.first),d,l=null,g,c=[],p=[],A,k,u,v;if(n)for(v=0;v<t;v+=1)A=r[v],k=f(A,v),u=s.get(k).e,(u.f&O)===0&&((L=(T=u.nodes)==null?void 0:T.a)==null||L.measure(),(g??(g=new Set)).add(u));for(v=0;v<t;v+=1){if(A=r[v],k=f(A,v),u=s.get(k).e,e.outrogroups!==null)for(const M of e.outrogroups)M.pending.delete(u),M.done.delete(u);if((u.f&$)!==0&&(ke(u),n&&((Y=(I=u.nodes)==null?void 0:I.a)==null||Y.unfix(),(g??(g=new Set)).delete(u))),(u.f&O)!==0)if(u.f^=O,u===o)V(u,null,a);else{var C=l?l.next:o;u===e.effect.last&&(e.effect.last=u.prev),u.prev&&(u.prev.next=u.next),u.next&&(u.next.prev=u.prev),m(e,l,u),m(e,u,C),V(u,C,a),l=u,c=[],p=[],o=z(l.next);continue}if(u!==o){if(d!==void 0&&d.has(u)){if(c.length<p.length){var E=p[0],b;l=E.prev;var R=c[0],_=c[c.length-1];for(b=0;b<c.length;b+=1)V(c[b],E,a);for(b=0;b<p.length;b+=1)d.delete(p[b]);m(e,R.prev,_.next),m(e,l,R),m(e,_,E),o=E,l=_,v-=1,c=[],p=[]}else d.delete(u),V(u,o,a),m(e,u.prev,u.next),m(e,u,l===null?e.effect.first:l.next),m(e,l,u),l=u;continue}for(c=[],p=[];o!==null&&o!==u;)(d??(d=new Set)).add(o),p.push(o),o=z(o.next);if(o===null)continue}(u.f&O)===0&&c.push(u),l=u,o=z(u.next)}if(e.outrogroups!==null){for(const M of e.outrogroups)M.pending.size===0&&(ae(e,fe(M.done)),(G=e.outrogroups)==null||G.delete(M));e.outrogroups.size===0&&(e.outrogroups=null)}if(o!==null||d!==void 0){var h=[];if(d!==void 0)for(u of d)(u.f&$)===0&&h.push(u);for(;o!==null;)(o.f&$)===0&&o!==e.fallback&&h.push(o),o=z(o.next);var N=h.length;if(N>0){var w=(i&Te)!==0&&t===0?a:null;if(n){for(v=0;v<N;v+=1)(ue=(se=h[v].nodes)==null?void 0:se.a)==null||ue.measure();for(v=0;v<N;v+=1)(oe=(le=h[v].nodes)==null?void 0:le.a)==null||oe.fix()}Wr(e,h,w)}}n&&Ce(()=>{var M,ce;if(g!==void 0)for(u of g)(ce=(M=u.nodes)==null?void 0:M.a)==null||ce.apply()})}function Ur(e,r,a,i,f,n,t,s){var o=(t&er)!==0?(t&rr)===0?ar(a,!1,!1):ge(a):null,d=(t&ir)!==0?ge(f):null;return{v:o,i:d,e:q(()=>(n(r,o??a,d??f,s),()=>{e.delete(i)}))}}function V(e,r,a){if(e.nodes)for(var i=e.nodes.start,f=e.nodes.end,n=r&&(r.f&O)===0?r.nodes.start:a;i!==null;){var t=lr(i);if(n.before(i),i===f)return;i=t}}function m(e,r,a){r===null?e.effect.first=a:r.next=a,a===null?e.effect.last=r:a.prev=r}function Vr(e,r,...a){var i=new Re(e);ie(()=>{const f=r()??null;i.ensure(f,f&&(n=>f(n,...a)))},Oe)}function Br(e,r,a,i,f,n){let t=S;S&&re();var s=null;S&&D.nodeType===or&&(s=D,re());var o=S?D:e,d=new Re(o,!1);ie(()=>{const l=r()||null;var g=dr;if(l===null){d.ensure(null,null);return}return d.ensure(l,c=>{if(l){if(s=S?s:cr(l,g),We(s,s),i){S&&ze(l)&&s.append(document.createComment(""));var p=S?Ne(s):s.appendChild(K());S&&(p===null?P(!1):H(p)),i(s,p)}vr.nodes.end=s,c.before(s)}S&&H(c)}),()=>{}},Oe),Le(()=>{}),t&&(P(!0),H(o))}function qr(e,r){var a=void 0,i;Me(()=>{a!==(a=r())&&(i&&(X(i),i=null),a&&(i=q(()=>{ne(()=>a(e))})))})}function me(e){var r,a,i="";if(typeof e=="string"||typeof e=="number")i+=e;else if(typeof e=="object")if(Array.isArray(e)){var f=e.length;for(r=0;r<f;r++)e[r]&&(a=me(e[r]))&&(i&&(i+=" "),i+=a)}else for(a in e)e[a]&&(i&&(i+=" "),i+=a);return i}function Yr(){for(var e,r,a=0,i="",f=arguments.length;a<f;a++)(e=arguments[a])&&(r=me(e))&&(i&&(i+=" "),i+=r);return i}function Gr(e){return typeof e=="object"?Yr(e):e??""}const pe=[...` 	
\r\f \v\uFEFF`];function Kr(e,r,a){var i=e==null?"":""+e;if(r&&(i=i?i+" "+r:r),a){for(var f of Object.keys(a))if(a[f])i=i?i+" "+f:f;else if(i.length)for(var n=f.length,t=0;(t=i.indexOf(f,t))>=0;){var s=t+n;(t===0||pe.includes(i[t-1]))&&(s===i.length||pe.includes(i[s]))?i=(t===0?"":i.substring(0,t))+i.substring(s+1):t=s}}return i===""?null:i}function Ae(e,r=!1){var a=r?" !important;":";",i="";for(var f of Object.keys(e)){var n=e[f];n!=null&&n!==""&&(i+=" "+f+": "+n+a)}return i}function x(e){return e[0]!=="-"||e[1]!=="-"?e.toLowerCase():e}function jr(e,r){if(r){var a="",i,f;if(Array.isArray(r)?(i=r[0],f=r[1]):i=r,e){e=String(e).replaceAll(/\s*\/\*.*?\*\/\s*/g,"").trim();var n=!1,t=0,s=!1,o=[];i&&o.push(...Object.keys(i).map(x)),f&&o.push(...Object.keys(f).map(x));var d=0,l=-1;const k=e.length;for(var g=0;g<k;g++){var c=e[g];if(s?c==="/"&&e[g-1]==="*"&&(s=!1):n?n===c&&(n=!1):c==="/"&&e[g+1]==="*"?s=!0:c==='"'||c==="'"?n=c:c==="("?t++:c===")"&&t--,!s&&n===!1&&t===0){if(c===":"&&l===-1)l=g;else if(c===";"||g===k-1){if(l!==-1){var p=x(e.substring(d,l).trim());if(!o.includes(p)){c!==";"&&g++;var A=e.substring(d,g).trim();a+=" "+A+";"}}d=g+1,l=-1}}}}return i&&(a+=Ae(i)),f&&(a+=Ae(f,!0)),a=a.trim(),a===""?null:a}return e==null?null:String(e)}function Xr(e,r,a,i,f,n){var t=e.__className;if(S||t!==a||t===void 0){var s=Kr(a,i,n);(!S||s!==e.getAttribute("class"))&&(s==null?e.removeAttribute("class"):r?e.className=s:e.setAttribute("class",s)),e.__className=a}else if(n&&f!==n)for(var o in n){var d=!!n[o];(f==null||d!==!!f[o])&&e.classList.toggle(o,d)}return n}function ee(e,r={},a,i){for(var f in a){var n=a[f];r[f]!==n&&(a[f]==null?e.style.removeProperty(f):e.style.setProperty(f,n,i))}}function Jr(e,r,a,i){var f=e.__style;if(S||f!==r){var n=jr(r,i);(!S||n!==e.getAttribute("style"))&&(n==null?e.removeAttribute("style"):e.style.cssText=n),e.__style=r}else i&&(Array.isArray(i)?(ee(e,a==null?void 0:a[0],i[0]),ee(e,a==null?void 0:a[1],i[1],"important")):ee(e,a,i));return i}function J(e,r,a=!1){if(e.multiple){if(r==null)return;if(!Ie(r))return gr();for(var i of e.options)i.selected=r.includes(B(i));return}for(i of e.options){var f=B(i);if(_r(f,r)){i.selected=!0;return}}(!a||r!==void 0)&&(e.selectedIndex=-1)}function ye(e){var r=new MutationObserver(()=>{J(e,e.__value)});r.observe(e,{childList:!0,subtree:!0,attributes:!0,attributeFilter:["value"]}),Le(()=>{r.disconnect()})}function ca(e,r,a=r){var i=new WeakSet,f=!0;hr(e,"change",n=>{var t=n?"[selected]":":checked",s;if(e.multiple)s=[].map.call(e.querySelectorAll(t),B);else{var o=e.querySelector(t)??e.querySelector("option:not([disabled])");s=o&&B(o)}a(s),e.__value=s,j!==null&&i.add(j)}),ne(()=>{var n=r();if(e===document.activeElement){var t=j;if(i.has(t))return}if(J(e,n,f),f&&n===void 0){var s=e.querySelector(":checked");s!==null&&(n=B(s),a(n))}e.__value=n,f=!1}),ye(e)}function B(e){return"__value"in e?e.__value:e.value}const F=Symbol("class"),U=Symbol("style"),De=Symbol("is custom element"),Pe=Symbol("is html"),Qr=Z?"link":"LINK",Zr=Z?"input":"INPUT",$r=Z?"option":"OPTION",xr=Z?"select":"SELECT";function ea(e){if(S){var r=!1,a=()=>{if(!r){if(r=!0,e.hasAttribute("value")){var i=e.value;Q(e,"value",null),e.value=i}if(e.hasAttribute("checked")){var f=e.checked;Q(e,"checked",null),e.checked=f}}};e.__on_r=a,Ce(a),Sr()}}function da(e,r){var a=te(e);a.checked!==(a.checked=r??void 0)&&(e.checked=r)}function ra(e,r){r?e.hasAttribute("selected")||e.setAttribute("selected",""):e.removeAttribute("selected")}function Q(e,r,a,i){var f=te(e);S&&(f[r]=e.getAttribute(r),r==="src"||r==="srcset"||r==="href"&&e.nodeName===Qr)||f[r]!==(f[r]=a)&&(r==="loading"&&(e[kr]=a),a==null?e.removeAttribute(r):typeof a!="string"&&He(e).includes(r)?e[r]=a:e.setAttribute(r,a))}function aa(e,r,a,i,f=!1,n=!1){if(S&&f&&e.nodeName===Zr){var t=e,s=t.type==="checkbox"?"defaultChecked":"defaultValue";s in a||ea(t)}var o=te(e),d=o[De],l=!o[Pe];let g=S&&d;g&&P(!1);var c=r||{},p=e.nodeName===$r;for(var A in r)A in a||(a[A]=null);a.class?a.class=Gr(a.class):a[F]&&(a.class=null),a[U]&&(a.style??(a.style=null));var k=He(e);for(const _ in a){let h=a[_];if(p&&_==="value"&&h==null){e.value=e.__value="",c[_]=h;continue}if(_==="class"){var u=e.namespaceURI==="http://www.w3.org/1999/xhtml";Xr(e,u,h,i,r==null?void 0:r[F],a[F]),c[_]=h,c[F]=a[F];continue}if(_==="style"){Jr(e,h,r==null?void 0:r[U],a[U]),c[_]=h,c[U]=a[U];continue}var v=c[_];if(!(h===v&&!(h===void 0&&e.hasAttribute(_)))){c[_]=h;var C=_[0]+_[1];if(C!=="$$")if(C==="on"){const N={},w="$$"+_;let T=_.slice(2);var E=Ye(T);if(Fe(T)&&(T=T.slice(0,-7),N.capture=!0),!E&&v){if(h!=null)continue;e.removeEventListener(T,c[w],N),c[w]=null}if(E)Ue(T,e,h),Ve([T]);else if(h!=null){let L=function(I){c[_].call(this,I)};c[w]=Be(T,e,L,N)}}else if(_==="style")Q(e,_,h);else if(_==="autofocus")Ar(e,!!h);else if(!d&&(_==="__value"||_==="value"&&h!=null))e.value=e.__value=h;else if(_==="selected"&&p)ra(e,h);else{var b=_;l||(b=qe(b));var R=b==="defaultValue"||b==="defaultChecked";if(h==null&&!d&&!R)if(o[_]=null,b==="value"||b==="checked"){let N=e;const w=r===void 0;if(b==="value"){let T=N.defaultValue;N.removeAttribute(b),N.defaultValue=T,N.value=N.__value=w?T:null}else{let T=N.defaultChecked;N.removeAttribute(b),N.defaultChecked=T,N.checked=w?T:!1}}else e.removeAttribute(_);else R||k.includes(b)&&(d||typeof h!="string")?(e[b]=h,b in o&&(o[b]=Er)):typeof h!="function"&&Q(e,b,h)}}}return g&&P(!0),c}function Ee(e,r,a=[],i=[],f=[],n,t=!1,s=!1){br(f,a,i,o=>{var d=void 0,l={},g=e.nodeName===xr,c=!1;if(Me(()=>{var A=r(...o.map(y)),k=aa(e,d,A,n,t,s);c&&g&&"value"in A&&J(e,A.value);for(let v of Object.getOwnPropertySymbols(l))A[v]||X(l[v]);for(let v of Object.getOwnPropertySymbols(A)){var u=A[v];v.description===pr&&(!d||u!==d[v])&&(l[v]&&X(l[v]),l[v]=q(()=>qr(e,()=>u))),k[v]=u}d=k}),g){var p=e;ne(()=>{J(p,d.value,!0),ye(p)})}c=!0})}function te(e){return e.__attributes??(e.__attributes={[De]:e.nodeName.includes("-"),[Pe]:e.namespaceURI===Tr})}var Se=new Map;function He(e){var r=e.getAttribute("is")||e.nodeName,a=Se.get(r);if(a)return a;Se.set(r,a=[]);for(var i,f=e,n=Element.prototype;n!==f;){i=wr(f);for(var t in i)i[t].set&&a.push(t);f=Nr(f)}return a}/**
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
 */const ia={xmlns:"http://www.w3.org/2000/svg",width:24,height:24,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":2,"stroke-linecap":"round","stroke-linejoin":"round"};/**
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
 */const fa=e=>{for(const r in e)if(r.startsWith("aria-")||r==="role"||r==="title")return!0;return!1};/**
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
 */const na=Symbol("lucide-context"),ta=()=>Cr(na);var sa=Ge("<svg><!><!></svg>");function va(e,r){Ir(r,!0);const a=ta()??{},i=W(r,"color",19,()=>a.color??"currentColor"),f=W(r,"size",19,()=>a.size??24),n=W(r,"strokeWidth",19,()=>a.strokeWidth??2),t=W(r,"absoluteStrokeWidth",19,()=>a.absoluteStrokeWidth??!1),s=W(r,"iconNode",19,()=>[]),o=Pr(r,["$$slots","$$events","$$legacy","name","color","size","strokeWidth","absoluteStrokeWidth","iconNode","children"]),d=_e(()=>t()?Number(n())*24/Number(f()):n());var l=sa();Ee(l,p=>({...ia,...p,...o,width:f(),height:f(),stroke:i(),"stroke-width":y(d),class:["lucide-icon lucide",a.class,r.name&&`lucide-${r.name}`,r.class]}),[()=>!r.children&&!fa(o)&&{"aria-hidden":"true"}]);var g=Lr(l);zr(g,17,s,Hr,(p,A)=>{var k=_e(()=>Dr(y(A),2));let u=()=>y(k)[0],v=()=>y(k)[1];var C=Ke(),E=Mr(C);Br(E,u,!0,(b,R)=>{Ee(b,()=>({...v()}))}),de(p,C)});var c=Rr(g);Vr(c,()=>r.children??mr),yr(l),de(e,l),Or()}export{va as I,Xr as a,ca as b,Q as c,ye as d,zr as e,J as f,Jr as g,da as h,Hr as i,Gr as j,ea as r,Vr as s};
