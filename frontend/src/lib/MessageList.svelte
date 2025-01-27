<script>
  import { fade } from "svelte/transition";
  import { apiClient } from "../js/api";

  let { selectedDevice } = $props();

  let page = $state(1);
  let messages = $state([]);
  let isLoading = $state(false);
  let error = $state(null);
  let totalPages = $state(0);
  const per_page = 10;

  $effect(() => {
    if (selectedDevice || page) {
      loadMessages();
    }
  });

  async function loadMessages() {
    isLoading = true;
    error = null;
    try {
      const response = await apiClient.getSmsPaginated(
        page,
        per_page,
        selectedDevice?.name,
      );

      if (response.status === 200) {
        messages = response.data.data || [];
        totalPages = Math.ceil(response.data.total / 10);
      }
    } catch (err) {
      error = err.message?.includes("Failed to fetch")
        ? "Network error: Unable to connect to server"
        : `Error loading messages: ${err.message}`;
    } finally {
      isLoading = false;
    }
  }

  function nextPage() {
    if (page < totalPages) page++;
  }

  function prevPage() {
    if (page > 1) page--;
  }
</script>

<div class="messages-container">
  {#if isLoading}
    <div class="status-message loading">
      <div class="spinner"></div>
      Loading messages...
    </div>
  {:else if error}
    <div class="status-message error">
      ⚠️ {error}
      <button onclick={loadMessages}>Retry</button>
    </div>
  {:else if messages.length === 0}
    <div class="status-message empty">📭 No messages found for this device</div>
  {:else}
    {#each messages as message (message.id)}
      <div transition:fade class="message-card {message.sender? 'sender-card':'receiver-card'}">
        <div class="sender-receiver">
          {#if message.sender}
            sender:{message.sender}
          {/if}
          {#if message.receiver}
            receiver:{message.receiver}
          {/if}
        </div>

        <div class="content">{message.message}</div>
        <div class="meta">
          <span class="device"
            >{#if !selectedDevice}
              {message.device}
            {/if}</span
          >
          <span class="timestamp">{message.timestamp}</span>
        </div>
      </div>
    {/each}

    <div class="pagination">
      <button onclick={prevPage} disabled={page === 1}> ← Previous </button>
      <span>Page {page} of {totalPages}</span>
      <button onclick={nextPage} disabled={page === totalPages}>
        Next →
      </button>
    </div>
  {/if}
</div>

<style>
  .messages-container {
    max-height: calc(100vh - 150px);
    overflow-y: auto;
    padding: 0 15px;
  }

  .message-card {
    padding: 1rem;
    margin: 1rem 0;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    
    transition: transform 0.2s ease;
  }

  .sender-card{
    border-left: 4px solid var(--primary-color);
  }

  .receiver-card{
    border-left: 4px solid var(--secondary-color);
  }

  .message-card:hover {
    transform: translateX(5px);
  }

  .content {
    font-size: 0.95rem;
    color: #333;
    margin-bottom: 0.5rem;
  }

  .meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: #666;
  }

  .timestamp {
    font-style: italic;
  }

  .status-message {
    padding: 2rem;
    text-align: center;
    border-radius: 8px;
    margin: 1rem 0;

    &.loading {
      color: #666;
    }

    &.error {
      color: #dc3545;
      background: #ffe6e6;
    }

    &.empty {
      color: #6c757d;
      background: #f8f9fa;
    }
  }

  .spinner {
    display: inline-block;
    width: 1.5rem;
    height: 1.5rem;
    border: 3px solid rgba(0, 0, 0, 0.1);
    border-top-color: #666;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-right: 0.5rem;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: #f8f9fa;
    border-radius: 8px;
    margin-top: 1rem;

    button {
      padding: 0.5rem 1rem;
      border: 1px solid #ddd;
      border-radius: 4px;
      background: white;
      cursor: pointer;

      &:hover:not(:disabled) {
        background: var(--primary-color);
        color: white;
      }

      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    }
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .sender-receiver {
    font-size: 0.7rem;
    margin-bottom: 0.7rem;
    color: #666;
  }
</style>
