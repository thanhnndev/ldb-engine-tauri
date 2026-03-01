<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from 'svelte';
  import type { Instance, CreateInstanceRequest, PullProgress } from '$lib/types';
  import InstanceCard from './InstanceCard.svelte';
  import InstanceForm from './InstanceForm.svelte';

  let instances = $state<Instance[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showForm = $state(false);
  let creating = $state(false);
  let operationLoading = $state<string | null>(null);

  // Progress tracking for image pulls
  let pullProgress = $state<PullProgress | null>(null);
  let creationStage = $state<string>('');
  let pullPercentage = $state<number>(0);

  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let unlistenProgress: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;

  // Helper function to add timeout to promises
  function withTimeout<T>(promise: Promise<T>, ms: number, errorMessage: string): Promise<T> {
    const timeout = new Promise<never>((_, reject) => {
      setTimeout(() => reject(new Error(errorMessage)), ms);
    });
    return Promise.race([promise, timeout]);
  }

  async function loadInstances() {
    try {
      instances = await withTimeout(
        invoke<Instance[]>("list_instances"),
        10000,
        'Loading instances timed out. Make sure Docker is running.'
      );
      error = null;
    } catch (e) {
      console.error("Failed to load instances:", e);
      const errorMsg = String(e);
      if (errorMsg.includes('Docker') || errorMsg.includes('docker')) {
        error = 'Failed to connect to Docker. Please ensure Docker is running.';
      } else {
        error = errorMsg;
      }
    } finally {
      loading = false;
    }
  }

  async function createInstance(request: CreateInstanceRequest) {
    creating = true;
    creationStage = 'pulling';
    pullProgress = null;
    pullPercentage = 0;
    error = null;

    try {
      const fullImage = `${request.image}:${request.tag}`;
      
      // First, pull the image (with progress events)
      creationStage = 'pulling';
      await invoke("pull_docker_image", { image: fullImage });
      
      // Then create the container
      creationStage = 'creating';
      const newInstance = await invoke<Instance>("create_instance", { request });
      
      instances = [...instances, newInstance];
      showForm = false;
      creationStage = '';
    } catch (e) {
      console.error("Failed to create instance:", e);
      error = String(e);
      creationStage = '';
    } finally {
      creating = false;
      pullProgress = null;
      pullPercentage = 0;
    }
  }

  async function startInstance(instance: Instance) {
    operationLoading = `start-${instance.id}`;
    try {
      const updated = await invoke<Instance>("start_instance", { containerId: instance.name });
      instances = instances.map(i => i.id === instance.id ? { ...i, status: updated.status } : i);
    } catch (e) {
      console.error("Failed to start instance:", e);
      error = String(e);
    } finally {
      operationLoading = null;
    }
  }

  async function stopInstance(instance: Instance) {
    operationLoading = `stop-${instance.id}`;
    try {
      const updated = await invoke<Instance>("stop_instance", { containerId: instance.name });
      instances = instances.map(i => i.id === instance.id ? { ...i, status: updated.status } : i);
    } catch (e) {
      console.error("Failed to stop instance:", e);
      error = String(e);
    } finally {
      operationLoading = null;
    }
  }

  async function restartInstance(instance: Instance) {
    operationLoading = `restart-${instance.id}`;
    try {
      const updated = await invoke<Instance>("restart_instance", { containerId: instance.name });
      instances = instances.map(i => i.id === instance.id ? { ...i, status: updated.status } : i);
    } catch (e) {
      console.error("Failed to restart instance:", e);
      error = String(e);
    } finally {
      operationLoading = null;
    }
  }

  async function deleteInstance(instance: Instance, deleteVolume: boolean = false) {
    operationLoading = `delete-${instance.id}`;
    try {
      await invoke("delete_instance", { containerId: instance.name, deleteVolume });
      instances = instances.filter(i => i.id !== instance.id);
    } catch (e) {
      console.error("Failed to delete instance:", e);
      error = String(e);
    } finally {
      operationLoading = null;
    }
  }

  function startPolling() {
    pollInterval = setInterval(loadInstances, 5000);
  }

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  async function setupEventListeners() {
    // Listen for pull progress events
    unlistenProgress = await listen<PullProgress>('pull-progress', (event) => {
      pullProgress = event.payload;
      // Calculate percentage if we have progress details
      if (event.payload.progress_detail?.current && event.payload.progress_detail?.total) {
        pullPercentage = Math.round(
          (event.payload.progress_detail.current / event.payload.progress_detail.total) * 100
        );
      }
    });

    unlistenComplete = await listen<string>('pull-complete', () => {
      pullProgress = null;
      pullPercentage = 100;
    });
  }

  onMount(() => {
    loadInstances();
    startPolling();
    setupEventListeners();
  });

  onDestroy(() => {
    stopPolling();
    if (unlistenProgress) unlistenProgress();
    if (unlistenComplete) unlistenComplete();
  });
</script>

<div class="instance-list">
  <div class="header">
    <h2>Instances</h2>
    <button class="new-btn" onclick={() => showForm = true}>
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New Instance
    </button>
  </div>

  {#if error}
    <div class="error-banner">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span>{error}</span>
      <button onclick={() => error = null}>Dismiss</button>
    </div>
  {/if}

  {#if showForm}
    <div class="form-overlay">
      <div class="form-container">
        {#if creating}
          <div class="progress-overlay">
            <div class="progress-content">
              <div class="progress-spinner"></div>
              <div class="progress-stage">
                {#if creationStage === 'pulling'}
                  <span class="stage-icon">📥</span>
                  <span>Pulling Docker image...</span>
                {:else if creationStage === 'creating'}
                  <span class="stage-icon">🐳</span>
                  <span>Creating container...</span>
                {:else}
                  <span class="stage-icon">⏳</span>
                  <span>Processing...</span>
                {/if}
              </div>
              {#if pullProgress && pullProgress.status}
                <div class="progress-details">
                  <p class="progress-status">{pullProgress.status}</p>
                  {#if pullProgress.progress}
                    <div class="progress-bar-container">
                      <div class="progress-bar" style="width: {Math.min(pullPercentage, 100)}%"></div>
                    </div>
                    <p class="progress-text">{pullProgress.progress}</p>
                  {/if}
                </div>
              {:else if creationStage === 'pulling'}
                <div class="progress-details">
                  <p class="progress-status">Downloading image layers...</p>
                  <div class="progress-bar-container">
                    <div class="progress-bar indeterminate"></div>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {:else}
          <InstanceForm 
            onsubmit={createInstance}
            oncancel={() => showForm = false}
            loading={creating}
          />
        {/if}
      </div>
    </div>
  {/if}

  {#if loading}
    <div class="loading">
      <span class="spinner"></span>
      Loading instances...
    </div>
  {:else if instances.length === 0}
    <div class="empty">
      <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
        <line x1="8" y1="21" x2="16" y2="21"/>
        <line x1="12" y1="17" x2="12" y2="21"/>
      </svg>
      <h3>No instances yet</h3>
      <p>Create your first database instance to get started.</p>
      <button class="new-btn-large" onclick={() => showForm = true}>
        Create Instance
      </button>
    </div>
  {:else}
    <div class="instances-grid">
      {#each instances as instance (instance.id)}
        <InstanceCard 
          {instance}
          onstart={() => startInstance(instance)}
          onstop={() => stopInstance(instance)}
          onrestart={() => restartInstance(instance)}
          ondelete={(deleteVolume) => deleteInstance(instance, deleteVolume)}
          loading={operationLoading?.startsWith('start-') && operationLoading?.endsWith(instance.id) ||
                  operationLoading?.startsWith('stop-') && operationLoading?.endsWith(instance.id) ||
                  operationLoading?.startsWith('restart-') && operationLoading?.endsWith(instance.id) ||
                  operationLoading?.startsWith('delete-') && operationLoading?.endsWith(instance.id)}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .instance-list {
    width: 100%;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: #1f2937;
  }

  .new-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    border: none;
    border-radius: 8px;
    background: #3b82f6;
    color: white;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .new-btn:hover {
    background: #2563eb;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 8px;
    color: #dc2626;
    margin-bottom: 1rem;
  }

  .error-banner button {
    margin-left: auto;
    padding: 0.25rem 0.75rem;
    border: none;
    border-radius: 4px;
    background: #dc2626;
    color: white;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .form-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }

  .form-container {
    width: 100%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
    background: white;
    border-radius: 12px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  }

  .progress-overlay {
    padding: 3rem 2rem;
    text-align: center;
  }

  .progress-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
  }

  .progress-spinner {
    width: 48px;
    height: 48px;
    border: 3px solid #e5e7eb;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .progress-stage {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 1.15rem;
    font-weight: 500;
    color: #1f2937;
  }

  .stage-icon {
    font-size: 1.5rem;
  }

  .progress-details {
    width: 100%;
    max-width: 350px;
    margin-top: 0.5rem;
  }

  .progress-status {
    font-size: 0.9rem;
    color: #6b7280;
    margin: 0 0 0.75rem;
    word-break: break-word;
  }

  .progress-bar-container {
    width: 100%;
    height: 8px;
    background: #e5e7eb;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6, #2563eb);
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .progress-bar.indeterminate {
    width: 30%;
    animation: indeterminate 1.5s infinite linear;
  }

  @keyframes indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(400%); }
  }

  .progress-text {
    font-size: 0.8rem;
    color: #9ca3af;
    margin: 0.5rem 0 0;
    font-family: monospace;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 3rem;
    color: #6b7280;
    font-size: 1rem;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid #e5e7eb;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty {
    text-align: center;
    padding: 4rem 2rem;
    background: #f9fafb;
    border: 2px dashed #e5e7eb;
    border-radius: 12px;
  }

  .empty svg {
    color: #9ca3af;
    margin-bottom: 1rem;
  }

  .empty h3 {
    margin: 0 0 0.5rem;
    font-size: 1.25rem;
    font-weight: 600;
    color: #374151;
  }

  .empty p {
    margin: 0 0 1.5rem;
    color: #6b7280;
  }

  .new-btn-large {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    background: #3b82f6;
    color: white;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .new-btn-large:hover {
    background: #2563eb;
  }

  .instances-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
