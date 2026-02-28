<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import type { PullProgress } from '$lib/types';

  interface Props {
    image: string;
    oncomplete: () => void;
    oncancel: () => void;
  }

  let { image, oncomplete, oncancel }: Props = $props();

  let progress: PullProgress[] = $state([]);
  let currentStatus = $state('Preparing...');
  let isComplete = $state(false);
  let isError = $state(false);
  let errorMessage = $state('');
  let unlisten: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;

  onMount(async () => {
    // Listen for pull progress events
    unlisten = await listen<PullProgress>('pull-progress', (event) => {
      const p = event.payload;
      progress = [...progress, p];
      currentStatus = p.status;
      
      if (p.progress) {
        currentStatus = `${p.status}: ${p.progress}`;
      }
    });

    // Listen for pull completion
    unlistenComplete = await listen<string>('pull-complete', () => {
      isComplete = true;
      currentStatus = 'Pull complete!';
      oncomplete();
    });

    // Listen for pull errors
    unlistenError = await listen<string>('pull-error', (event) => {
      isError = true;
      errorMessage = event.payload;
      currentStatus = 'Error';
    });
  });

  onDestroy(() => {
    unlisten?.();
    unlistenComplete?.();
    unlistenError?.();
  });
</script>

<div class="pull-progress">
  <div class="header">
    <h4>Pulling {image}</h4>
    {#if !isComplete && !isError}
      <button class="cancel-btn" onclick={oncancel}>Cancel</button>
    {/if}
  </div>
  
  <div class="status" class:error={isError} class:complete={isComplete}>
    {#if isError}
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="15" y1="9" x2="9" y2="15"/>
        <line x1="9" y1="9" x2="15" y2="15"/>
      </svg>
    {:else if isComplete}
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
        <polyline points="22 4 12 14.01 9 11.01"/>
      </svg>
    {:else}
      <span class="spinner"></span>
    {/if}
    <span>{currentStatus}</span>
  </div>
  
  {#if isError}
    <p class="error-message">{errorMessage}</p>
  {/if}
  
  {#if progress.length > 0 && !isComplete && !isError}
    <div class="progress-details">
      {#each progress.slice(-5) as p}
        <div class="progress-item">
          <span class="layer">{p.id || 'Layer'}</span>
          <span class="layer-status">{p.status}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .pull-progress {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 1rem;
    background: white;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }
  
  h4 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: #1f2937;
  }
  
  .cancel-btn {
    padding: 0.25rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: white;
    font-size: 0.85rem;
    color: #6b7280;
    cursor: pointer;
  }
  
  .cancel-btn:hover {
    background: #f3f4f6;
  }
  
  .status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: #f9fafb;
    border-radius: 6px;
    font-size: 0.9rem;
    color: #374151;
  }
  
  .status.complete {
    background: #ecfdf5;
    color: #059669;
  }
  
  .status.error {
    background: #fef2f2;
    color: #dc2626;
  }
  
  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid #e5e7eb;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .error-message {
    margin: 0.5rem 0 0;
    font-size: 0.85rem;
    color: #dc2626;
  }
  
  .progress-details {
    margin-top: 0.75rem;
    font-size: 0.8rem;
    color: #6b7280;
  }
  
  .progress-item {
    display: flex;
    justify-content: space-between;
    padding: 0.25rem 0;
    border-bottom: 1px solid #f3f4f6;
  }
  
  .progress-item:last-child {
    border-bottom: none;
  }
  
  .layer {
    font-weight: 500;
  }
</style>
