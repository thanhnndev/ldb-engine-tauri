<script lang="ts">
  import type { InstanceStatus } from '$lib/types';

  interface Props {
    status: InstanceStatus;
    onstart: () => void;
    onstop: () => void;
    onrestart: () => void;
    ondelete: (deleteVolume: boolean) => void;
    loading?: boolean;
  }

  let { status, onstart, onstop, onrestart, ondelete, loading = false }: Props = $props();

  let showDeleteConfirm = $state(false);
  let deleteWithVolume = $state(false);

  function handleDelete() {
    if (showDeleteConfirm) {
      ondelete(deleteWithVolume);
      showDeleteConfirm = false;
      deleteWithVolume = false;
    } else {
      showDeleteConfirm = true;
    }
  }

  function cancelDelete() {
    showDeleteConfirm = false;
    deleteWithVolume = false;
  }

  const isRunning = $derived(status === 'running');
  const isStopped = $derived(status === 'stopped');
  const isLoading = $derived(loading);
</script>

<div class="controls">
  {#if isRunning}
    <button class="btn stop" onclick={onstop} disabled={isLoading}>
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="6" y="4" width="4" height="16"/>
        <rect x="14" y="4" width="4" height="16"/>
      </svg>
      Stop
    </button>
    <button class="btn restart" onclick={onrestart} disabled={isLoading}>
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M23 4v6h-6"/>
        <path d="M1 20v-6h6"/>
        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"/>
        <path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"/>
      </svg>
      Restart
    </button>
  {:else if isStopped}
    <button class="btn start" onclick={onstart} disabled={isLoading}>
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" stroke="none">
        <polygon points="5 3 19 12 5 21 5 3"/>
      </svg>
      Start
    </button>
  {/if}

  {#if showDeleteConfirm}
    <div class="delete-confirm">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={deleteWithVolume} />
        Delete volume data
      </label>
      <div class="confirm-actions">
        <button class="btn cancel" onclick={cancelDelete}>Cancel</button>
        <button class="btn delete" onclick={() => handleDelete()}>Confirm Delete</button>
      </div>
    </div>
  {:else}
    <button class="btn delete-outline" onclick={handleDelete} disabled={isLoading}>
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="3 6 5 6 21 6"/>
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
      </svg>
      Delete
    </button>
  {/if}
</div>

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: flex-start;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.875rem;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: none;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.start {
    background: #22c55e;
    color: white;
  }

  .btn.start:hover:not(:disabled) {
    background: #16a34a;
  }

  .btn.stop {
    background: #f59e0b;
    color: white;
  }

  .btn.stop:hover:not(:disabled) {
    background: #d97706;
  }

  .btn.restart {
    background: #3b82f6;
    color: white;
  }

  .btn.restart:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn.delete-outline {
    background: transparent;
    color: #ef4444;
    border: 1px solid #ef4444;
  }

  .btn.delete-outline:hover:not(:disabled) {
    background: #fef2f2;
  }

  .btn.delete {
    background: #ef4444;
    color: white;
  }

  .btn.delete:hover:not(:disabled) {
    background: #dc2626;
  }

  .btn.cancel {
    background: #f3f4f6;
    color: #374151;
  }

  .btn.cancel:hover:not(:disabled) {
    background: #e5e7eb;
  }

  .delete-confirm {
    width: 100%;
    margin-top: 0.5rem;
    padding: 0.75rem;
    background: #fef2f2;
    border-radius: 8px;
    border: 1px solid #fecaca;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #374151;
    cursor: pointer;
    margin-bottom: 0.5rem;
  }

  .confirm-actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
