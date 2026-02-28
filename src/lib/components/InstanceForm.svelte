<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SUPPORTED_IMAGES, type ImageTag, type DatabaseType, type CreateInstanceRequest } from '$lib/types';

  interface Props {
    onsubmit: (request: CreateInstanceRequest) => void;
    oncancel: () => void;
    loading?: boolean;
  }

  let { onsubmit, oncancel, loading = false }: Props = $props();

  let name = $state('');
  let databaseType = $state<DatabaseType>('postgres');
  let imageTag = $state('');
  let password = $state('');
  let port = $state<number | undefined>(undefined);

  let tags = $state<ImageTag[]>([]);
  let loadingTags = $state(false);
  let errors = $state<Record<string, string>>({});

  async function loadTags() {
    const image = SUPPORTED_IMAGES.find(img => img.id === databaseType);
    if (!image) return;

    loadingTags = true;
    tags = [];
    imageTag = '';
    
    try {
      tags = await invoke<ImageTag[]>("get_docker_tags", { image: image.hubName });
      // Set default tag to latest or first available
      const latestTag = tags.find(t => t.name === 'latest');
      imageTag = latestTag ? latestTag.name : (tags[0]?.name || '');
    } catch (e) {
      console.error("Failed to load tags:", e);
    } finally {
      loadingTags = false;
    }
  }

  function handleTypeChange() {
    loadTags();
    // Auto-suggest port based on database type
    const image = SUPPORTED_IMAGES.find(img => img.id === databaseType);
    if (image) {
      port = image.default_port;
    }
    // Clear image tag when type changes
    imageTag = '';
  }

  function validate(): boolean {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    } else if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
      newErrors.name = 'Name can only contain letters, numbers, underscores, and hyphens';
    }

    if (!imageTag) {
      newErrors.imageTag = 'Please select a version';
    }

    if (!password) {
      newErrors.password = 'Password is required';
    } else if (password.length < 4) {
      newErrors.password = 'Password must be at least 4 characters';
    }

    if (port && (port < 1024 || port > 65535)) {
      newErrors.port = 'Port must be between 1024 and 65535';
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  function handleSubmit() {
    if (!validate()) return;

    const image = SUPPORTED_IMAGES.find(img => img.id === databaseType);
    if (!image) return;

    const request: CreateInstanceRequest = {
      name: name.trim(),
      database_type: databaseType,
      image: image.hubName,
      tag: imageTag,
      password,
      port: port || undefined
    };

    onsubmit(request);
  }

  // Load tags on mount
  $effect(() => {
    loadTags();
  });
</script>

<form class="instance-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <h3>Create New Instance</h3>

  <div class="form-group">
    <label for="name">Instance Name</label>
    <input 
      type="text" 
      id="name" 
      bind:value={name}
      placeholder="my-database"
      disabled={loading}
      class:error={errors.name}
    />
    {#if errors.name}
      <span class="error-message">{errors.name}</span>
    {/if}
  </div>

  <div class="form-group">
    <label for="databaseType">Database Type</label>
    <select 
      id="databaseType" 
      bind:value={databaseType}
      onchange={handleTypeChange}
      disabled={loading}
    >
      {#each SUPPORTED_IMAGES as image}
        <option value={image.id}>{image.name}</option>
      {/each}
    </select>
  </div>

  <div class="form-group">
    <label for="imageTag">Version</label>
    {#if loadingTags}
      <div class="loading-tags">
        <span class="spinner"></span>
        Loading versions...
      </div>
    {:else}
      <select 
        id="imageTag" 
        bind:value={imageTag}
        disabled={loading || tags.length === 0}
        class:error={errors.imageTag}
      >
        <option value="">Select version</option>
        {#each tags as tag}
          <option value={tag.name}>{tag.name}</option>
        {/each}
      </select>
    {/if}
    {#if errors.imageTag}
      <span class="error-message">{errors.imageTag}</span>
    {/if}
  </div>

  <div class="form-group">
    <label for="password">
      {#if databaseType === 'redis'}
        Redis Password
      {:else if databaseType === 'mongo'}
        MongoDB Root Password
      {:else}
        Root Password
      {/if}
    </label>
    <input 
      type="password" 
      id="password" 
      bind:value={password}
      placeholder="Enter password"
      disabled={loading}
      class:error={errors.password}
    />
    {#if errors.password}
      <span class="error-message">{errors.password}</span>
    {/if}
  </div>

  <div class="form-group">
    <label for="port">Port (optional)</label>
    <input 
      type="number" 
      id="port" 
      bind:value={port}
      placeholder="Auto-detect"
      min="1024"
      max="65535"
      disabled={loading}
      class:error={errors.port}
    />
    <span class="hint">Leave empty for auto-detection</span>
    {#if errors.port}
      <span class="error-message">{errors.port}</span>
    {/if}
  </div>

  <div class="form-actions">
    <button type="button" class="btn cancel" onclick={oncancel} disabled={loading}>
      Cancel
    </button>
    <button type="submit" class="btn submit" disabled={loading}>
      {#if loading}
        <span class="spinner"></span>
        Creating...
      {:else}
        Create Instance
      {/if}
    </button>
  </div>
</form>

<style>
  .instance-form {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 1.5rem;
  }

  h3 {
    margin: 0 0 1.5rem;
    font-size: 1.25rem;
    font-weight: 600;
    color: #1f2937;
  }

  .form-group {
    margin-bottom: 1.25rem;
  }

  label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
    margin-bottom: 0.5rem;
  }

  input, select {
    width: 100%;
    padding: 0.625rem 0.875rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    font-size: 0.95rem;
    color: #1f2937;
    background: #f9fafb;
    transition: all 0.15s ease;
    box-sizing: border-box;
  }

  input:focus, select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
    background: white;
  }

  input.error, select.error {
    border-color: #ef4444;
  }

  input:disabled, select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .hint {
    display: block;
    font-size: 0.8rem;
    color: #6b7280;
    margin-top: 0.25rem;
  }

  .error-message {
    display: block;
    font-size: 0.8rem;
    color: #ef4444;
    margin-top: 0.25rem;
  }

  .loading-tags {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 0.875rem;
    color: #6b7280;
    font-size: 0.9rem;
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

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid #f3f4f6;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: none;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn.cancel {
    background: #f3f4f6;
    color: #374151;
  }

  .btn.cancel:hover:not(:disabled) {
    background: #e5e7eb;
  }

  .btn.submit {
    background: #3b82f6;
    color: white;
  }

  .btn.submit:hover:not(:disabled) {
    background: #2563eb;
  }
</style>
