<script lang="ts">
  // État des paramètres par générateur
  let entityParams = $state({ countries: ['US', 'FR'], seed: 42 });
  let contractParams = $state({ count: 10000, seed: 42 });
  let rateParams = $state({ curves: ['SOFR', 'ESTR'], historyDays: 1095 });
  let scheduleParams = $state({ types: ['nmd'], months: 120 });
  
  // État des logs
  let logs = $state('');
  let isRunning = $state(false);
  
  // Fonctions
  async function runGenerator(type: string, params: object) {
    isRunning = true;
    logs = `Lancement générateur ${type}...\n`;
    // Appel API backend
    const response = await fetch('/api/datasets/generate', {
      method: 'POST',
      body: JSON.stringify({ type, params })
    });
    const result = await response.json();
    logs += result.output;
    isRunning = false;
  }
</script>
<div class="tab-content">
  <div class="tab-header">
    <h2>Datasets Creator</h2>
  </div>
  
  <div class="generators-grid">
    <!-- Card: Entities -->
    <div class="generator-card">
      <h4>Entities Generator</h4>
      <label>
        Pays
        <select bind:value={entityParams.countries} multiple>
          <option value="US">United States</option>
          <option value="FR">France</option>
          <option value="ES">Spain</option>
          <option value="DE">Germany</option>
        </select>
      </label>
      <label>
        Seed
        <input type="number" bind:value={entityParams.seed} />
      </label>
      <button 
        class="btn-primary" 
        onclick={() => runGenerator('entities', entityParams)}
        disabled={isRunning}
      >
        Générer
      </button>
    </div>
    
    <!-- Card: Contracts -->
    <div class="generator-card">
      <h4>Contracts Generator</h4>
      <label>
        Nombre de contrats
        <input type="number" bind:value={contractParams.count} />
      </label>
      <label>
        Seed
        <input type="number" bind:value={contractParams.seed} />
      </label>
      <button 
        class="btn-primary" 
        onclick={() => runGenerator('contracts', contractParams)}
        disabled={isRunning}
      >
        Générer
      </button>
    </div>
    
    <!-- Card: Rate Series -->
    <div class="generator-card">
      <h4>Rate Series Generator</h4>
      ...
    </div>
    
    <!-- Card: Schedules -->
    <div class="generator-card">
      <h4>Schedules Generator</h4>
      ...
    </div>
  </div>
  
  <!-- Zone de logs -->
  <div class="logs-container">
    <h4>Logs de génération</h4>
    <pre>{logs}</pre>
  </div>
</div>