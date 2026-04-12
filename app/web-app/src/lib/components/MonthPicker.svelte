<script lang="ts">
  /**
   * Sélecteur mois/année.
   * value : string YYYY-MM (bindable, vide = aucune sélection)
   */
  let { value = $bindable(''), required = false } = $props();

  const MONTHS = [
    { v: 1,  l: 'Janvier'   },
    { v: 2,  l: 'Février'   },
    { v: 3,  l: 'Mars'      },
    { v: 4,  l: 'Avril'     },
    { v: 5,  l: 'Mai'       },
    { v: 6,  l: 'Juin'      },
    { v: 7,  l: 'Juillet'   },
    { v: 8,  l: 'Août'      },
    { v: 9,  l: 'Septembre' },
    { v: 10, l: 'Octobre'   },
    { v: 11, l: 'Novembre'  },
    { v: 12, l: 'Décembre'  },
  ];

  const THIS_YEAR = new Date().getFullYear();
  const YEARS = Array.from({ length: 41 }, (_, i) => THIS_YEAR - 10 + i); // -10 … +30

  let selYear  = $derived(value ? parseInt(value.slice(0, 4)) : 0);
  let selMonth = $derived(value ? parseInt(value.slice(5, 7)) : 0);

  function onYearChange(e: Event) {
    const y = parseInt((e.target as HTMLSelectElement).value);
    if (y && selMonth) value = `${y}-${String(selMonth).padStart(2, '0')}`;
    else if (y)        value = `${y}-01`;   // mois par défaut si seul l'année est choisie
    else               value = '';
  }

  function onMonthChange(e: Event) {
    const m = parseInt((e.target as HTMLSelectElement).value);
    if (selYear && m) value = `${selYear}-${String(m).padStart(2, '0')}`;
    else              value = '';
  }
</script>

<div class="mp">
  <select class="mp-select" onchange={onYearChange} value={selYear || ''}>
    <option value="">Année</option>
    {#each YEARS as y}
      <option value={y}>{y}</option>
    {/each}
  </select>
  <select class="mp-select" onchange={onMonthChange} value={selMonth || ''}>
    <option value="">Mois</option>
    {#each MONTHS as m}
      <option value={m.v}>{m.l}</option>
    {/each}
  </select>
</div>

<style>
  .mp {
    display: flex;
    gap: 6px;
  }
  .mp-select {
    flex: 1;
    min-width: 0;
  }
</style>
