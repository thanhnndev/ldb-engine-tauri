<script lang="ts">
  import type { ImageTag } from '$lib/types';

  interface Props {
    tags: ImageTag[];
    selectedTag: string | null;
    loading: boolean;
    onselect: (tag: string) => void;
  }

  let { tags, selectedTag, loading, onselect }: Props = $props();
</script>

<div class="tag-list">
  <h4>Available Versions</h4>
  
  {#if loading}
    <div class="loading">
      <span class="spinner"></span>
      Loading tags...
    </div>
  {:else if tags.length === 0}
    <p class="empty">No tags available</p>
  {:else}
    <div class="tags">
      {#each tags as tag}
        <button
          class="tag"
          class:selected={selectedTag === tag.name}
          onclick={() => onselect(tag.name)}
        >
          {tag.name}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tag-list {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 1rem;
    background: white;
  }
  
  h4 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    font-weight: 600;
    color: #374151;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  
  .loading {
    display: flex;
    align-items: center;
    gap: 0.5rem;
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
  
  .empty {
    color: #9ca3af;
    font-size: 0.9rem;
    margin: 0;
  }
  
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    max-height: 200px;
    overflow-y: auto;
  }
  
  .tag {
    padding: 0.35rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #f9fafb;
    font-size: 0.85rem;
    color: #374151;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  
  .tag:hover {
    border-color: #3b82f6;
    background: #eff6ff;
  }
  
  .tag.selected {
    border-color: #2563eb;
    background: #3b82f6;
    color: white;
  }
</style>
