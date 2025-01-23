<script>
    import { fade } from "svelte/transition";
    export let messages;
  </script>
  
  <div class="messages-container">
    {#each messages as message (message.id)}
      <div transition:fade class="message-card">
        <div>{message.content}</div>
        <div class="timestamp">{message.timestamp}</div>
      </div>
    {/each}
  </div>
  
  <style>
    .messages-container {
      max-height: calc(100vh - 150px);
      overflow-y: auto;
      padding-right: 10px;
    }
  
    .message-card {
      padding: 16px;
      margin: 12px 0;
      background: #f8f9fa;
      border-radius: 6px;
      border-left: 4px solid var(--primary-color);
      transition: box-shadow 0.2s;
    }
  
    .message-card:hover {
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }
  
    .timestamp {
      font-size: 0.8em;
      color: #6c757d;
      margin-top: 8px;
    }
  </style>