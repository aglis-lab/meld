<script lang="ts">
  let { title = "Svelte Counter" } = $props();

  let count = $state(0);

  const increment = () => count++;
  const decrement = () => count--;
  const reset = () => (count = 0);
</script>

<div class="counter-container">
  <h1>{title}</h1>
  <div class="counter-display">{count}</div>
  <div class="button-group">
    <button onclick={decrement}>-</button>
    <button onclick={increment}>+</button>
    <button onclick={reset}>Reset</button>
  </div>
</div>

<style>
  .counter-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 2rem;
    border: 2px solid #333;
    border-radius: 8px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    max-width: 300px;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .counter-display {
    font-size: 3rem;
    font-weight: bold;
    min-width: 80px;
    text-align: center;
  }

  .button-group {
    display: flex;
    gap: 0.5rem;
  }

  button {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: bold;
    border: none;
    border-radius: 4px;
    background-color: white;
    color: #667eea;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  button:hover {
    background-color: #f0f0f0;
    transform: scale(1.05);
  }

  button:active {
    transform: scale(0.95);
  }
</style>
