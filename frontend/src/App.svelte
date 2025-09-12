<script>
  import { fly } from "svelte/transition";
  import { quartOut } from "svelte/easing";
  import { isAuthenticated, isAuthLoading } from "./stores/auth";
  import { initConversation } from "./stores/conversation";
  import Login from "./pages/Login.svelte";
  import Dashboard from "./pages/Dashboard.svelte";


  // 监听认证状态变化
  $effect(() => {
    if ($isAuthenticated) {
      initConversation();
    }
  });
</script>

<!-- 应用容器 -->
<div class="app-container">
  {#if !$isAuthLoading}
    <!-- 页面内容 - 认证完成后直接显示 -->
    <div class="h-screen w-screen overflow-hidden">
      {#if $isAuthenticated}
        <!-- 主应用界面 -->
        <div 
          in:fly={{ x: 50, duration: 500, easing: quartOut }}
          out:fly={{ x: -50, duration: 300, easing: quartOut }}
        >
          <Dashboard />
        </div>
      {:else}
        <!-- 登录界面 -->
        <div 
          in:fly={{ x: -50, duration: 500, easing: quartOut }}
          out:fly={{ x: 50, duration: 300, easing: quartOut }}
        >
          <Login />
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .app-container {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  /* 自定义滚动条 */
  :global(::-webkit-scrollbar) {
    width: 6px;
    height: 6px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: rgba(148, 163, 184, 0.3);
    border-radius: 3px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(148, 163, 184, 0.5);
  }

  /* 暗色模式滚动条 */
  :global(.dark ::-webkit-scrollbar-thumb) {
    background: rgba(71, 85, 105, 0.3);
  }

  :global(.dark ::-webkit-scrollbar-thumb:hover) {
    background: rgba(71, 85, 105, 0.5);
  }
</style>
