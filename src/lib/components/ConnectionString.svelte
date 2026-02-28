<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';

  interface Props {
    instanceId: string;
    instanceName: string;
    databaseType: string;
    port: number;
    status: string;
  }

  let { instanceId, instanceName, databaseType, port, status }: Props = $props();

  let connectionString = $state<string | null>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);
  let copyFeedback = $state<string | null>(null);

  // Fetch connection string when component mounts and instance is running
  $effect(() => {
    if (status === 'running') {
      fetchConnectionString();
    }
  });

  async function fetchConnectionString() {
    isLoading = true;
    error = null;
    try {
      const result = await invoke<string>('get_connection_string', { instanceId });
      connectionString = result;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isLoading = false;
    }
  }

  async function copyToClipboard() {
    if (!connectionString) return;
    
    try {
      await writeText(connectionString);
      copyFeedback = 'Copied!';
      setTimeout(() => {
        copyFeedback = null;
      }, 2000);
    } catch (e) {
      copyFeedback = 'Failed to copy';
      setTimeout(() => {
        copyFeedback = null;
      }, 2000);
    }
  }
</script>

{#if status === 'running'}
  <div class="connection-string">
    <div class="header">
      <span class="label">Connection String</span>
      <button 
        class="copy-btn" 
        onclick={copyToClipboard}
        disabled={!connectionString}
        title="Copy to clipboard"
      >
        {#if copyFeedback}
          <span class="feedback">{copyFeedback}</span>
        {:else}
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
          <span>Copy</span>
        {/if}
      </button>
    </div>
    
    {#if isLoading}
      <div class="value loading">Loading...</div>
    {:else if error}
      <div class="value error">{error}</div>
    {:else if connectionString}
      <div class="value">{connectionString}</div>
    {:else}
      <div class="value empty">No connection string available</div>
    {/if}
  </div>
{/if}

<style>
  .connection-string {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: #f9fafb;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .label {
    font-size: 0.8rem;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .copy-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    font-size: 0.8rem;
    color: #374151;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .copy-btn:hover:not(:disabled) {
    background: #f3f4f6;
    border-color: #d1d5db;
  }

  .copy-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .copy-btn .feedback {
    color: #22c55e;
    font-weight: 500;
  }

  .value {
    font-family: monospace;
    font-size: 0.85rem;
    color: #374151;
    word-break: break-all;
    line-height: 1.4;
  }

  .value.loading {
    color: #9ca3af;
    font-style: italic;
  }

  .value.error {
    color: #ef4444;
  }

  .value.empty {
    color: #9ca3af;
    font-style: italic;
  }
</style>
