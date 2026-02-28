<script lang="ts">
  import type { SupportedImage } from '$lib/types';

  interface Props {
    image: SupportedImage;
    selected: boolean;
    loading: boolean;
    ontoggle: () => void;
  }

  let { image, selected, loading, ontoggle }: Props = $props();
</script>

<button
  class="image-card"
  class:selected
  onclick={ontoggle}
  disabled={loading}
>
  <div class="icon">
    {#if image.id === 'postgres'}
      <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <ellipse cx="12" cy="5" rx="9" ry="3"/>
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
      </svg>
    {:else if image.id === 'redis'}
      <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/>
        <path d="M8 12a2 2 0 1 0 4 0 2 2 0 0 0-4 0z"/>
      </svg>
    {:else if image.id === 'mysql'}
      <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2L2 7l10 5 10-5-10-5z"/>
        <path d="M2 17l10 5 10-5"/>
        <path d="M2 12l10 5 10-5"/>
      </svg>
    {:else}
      <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 17l6-6-6-6"/>
        <path d="M12 17h8"/>
      </svg>
    {/if}
  </div>
  <h3>{image.name}</h3>
  <p>{image.description}</p>
</button>

<style>
  .image-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1.5rem;
    border: 2px solid #e5e7eb;
    border-radius: 12px;
    background: white;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 160px;
  }
  
  .image-card:hover:not(:disabled) {
    border-color: #3b82f6;
    transform: translateY(-2px);
  }
  
  .image-card.selected {
    border-color: #2563eb;
    background: #eff6ff;
  }
  
  .image-card:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .icon {
    margin-bottom: 0.5rem;
    color: #3b82f6;
  }
  
  h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #1f2937;
  }
  
  p {
    margin: 0.25rem 0 0;
    font-size: 0.85rem;
    color: #6b7280;
    text-align: center;
  }
</style>
