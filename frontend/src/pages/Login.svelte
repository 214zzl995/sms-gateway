<script>
    import Icon from "@iconify/svelte";
    import { fade, fly } from 'svelte/transition'; 
    import { quintOut } from 'svelte/easing';
    import { updateStorageValue } from '../js/storage';

    let username = "";
    let password = "";
    let error = "";
    let isLoading = false;
    let showPassword = false;

    const handleSubmit = (/** @type {{ preventDefault: () => void; }} */ e) => {
        e.preventDefault();
        handleLogin();
    };

    const handleLogin = async () => {
        if (!username || !password) {
            error = "Username and password are required";
            return;
        }

        try {
            isLoading = true;
            error = "";

            const authToken = btoa(`${username}:${password}`);

            const response = await fetch("/api/check", {
                method: "GET",
                headers: {
                    Authorization: `Basic ${authToken}`,
                    "Content-Type": "application/json",
                },
            });

            switch (response.status) {
                case 204:
                    await updateStorageValue("auth", {
                        username,
                        token: authToken,
                    });
                    window.location.reload();
                    break;

                case 401:
                    error = "Invalid credentials";
                    break;

                default:
                    error = `Unexpected error: HTTP ${response.status}`;
                    break;
            }
        } catch (err) {
            error = err.message.includes("Failed to fetch")
                ? "Unable to connect to the server"
                : "Authentication process failed";
        } finally {
            isLoading = false;
        }
    };

    function togglePassword() {
        showPassword = !showPassword;
    }
</script>

<!-- Logo and Brand -->
<div class="fixed top-6 left-6 sm:top-8 sm:left-8 flex items-center gap-3 z-10">
    <div class="w-10 h-10 bg-gray-900 dark:bg-gray-100 rounded-lg flex items-center justify-center">
        <Icon icon="carbon:send-filled" class="w-5 h-5 text-gray-100 dark:text-gray-900" />
    </div>
    <div>
        <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">SMS Gateway</h1>
        <p class="text-xs text-gray-500 dark:text-gray-400">Secure messaging platform</p>
    </div>
</div>

<!-- Main Container -->
<div class="min-h-screen w-screen flex items-center justify-center bg-gray-50 dark:bg-zinc-900 transition-colors duration-300 px-4 sm:px-6 lg:px-8">

    <!-- Login Form -->
    <form
        class="relative w-full max-w-md mx-auto"
        onsubmit={handleSubmit}
        autocomplete="off"
        in:fly={{ y: 20, duration: 400, easing: quintOut }}
    >
        <!-- Card Container -->
        <div class="bg-white dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700 rounded-lg shadow-lg p-8 sm:p-10">
            <!-- Header -->
            <div class="text-center mb-8">
                <div class="w-16 h-16 bg-gray-900 dark:bg-gray-100 rounded-lg flex items-center justify-center mx-auto mb-4">
                    <Icon icon="carbon:user-avatar-filled" class="w-8 h-8 text-gray-100 dark:text-gray-900" />
                </div>
                <h2 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">
                    Welcome back
                </h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
                    Sign in to continue to SMS Gateway
                </p>
            </div>

            <!-- Error Message -->
            {#if error}
                <div
                    class="mb-6 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800/50 rounded-xl"
                    role="alert"
                    in:fade={{ duration: 200 }}
                >
                    <div class="flex items-center gap-3">
                        <Icon icon="carbon:warning-filled" class="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0" />
                        <p class="text-sm font-medium text-red-700 dark:text-red-300">{error}</p>
                    </div>
                </div>
            {/if}

            <!-- Form Fields -->
            <div class="space-y-5">
                <!-- Username Field -->
                <div>
                    <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Username
                    </label>
                    <div class="relative">
                        <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                            <Icon icon="carbon:user" class="w-5 h-5 text-gray-400 dark:text-gray-500" />
                        </div>
                        <input
                            id="username"
                            type="text"
                            bind:value={username}
                            placeholder="Enter your username"
                            class="w-full pl-11 pr-4 py-3 bg-white dark:bg-zinc-900
                                   border border-gray-300 dark:border-zinc-600 rounded-lg
                                   text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500
                                   transition-all duration-200
                                   focus:outline-none focus:border-gray-500 dark:focus:border-zinc-500
                                   hover:border-gray-400 dark:hover:border-zinc-500
                                   disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled={isLoading}
                            required
                        />
                    </div>
                </div>

                <!-- Password Field -->
                <div>
                    <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Password
                    </label>
                    <div class="relative">
                        <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                            <Icon icon="carbon:locked" class="w-5 h-5 text-gray-400 dark:text-gray-500" />
                        </div>
                        <input
                            id="password"
                            type={showPassword ? "text" : "password"}
                            bind:value={password}
                            placeholder="Enter your password"
                            class="w-full pl-11 pr-12 py-3 bg-white dark:bg-zinc-900
                                   border border-gray-300 dark:border-zinc-600 rounded-lg
                                   text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500
                                   transition-all duration-200
                                   focus:outline-none focus:border-gray-500 dark:focus:border-zinc-500
                                   hover:border-gray-400 dark:hover:border-zinc-500
                                   disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled={isLoading}
                            required
                        />
                        <button
                            type="button"
                            onclick={togglePassword}
                            class="absolute inset-y-0 right-0 pr-4 flex items-center"
                            tabindex="-1"
                        >
                            <Icon 
                                icon={showPassword ? "carbon:view-off" : "carbon:view"} 
                                class="w-5 h-5 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-400 transition-colors" 
                            />
                        </button>
                    </div>
                </div>

                <!-- Remember me -->
                <div class="flex items-center">
                    <label class="flex items-center gap-2 cursor-pointer">
                        <input 
                            type="checkbox" 
                            class="w-4 h-4 rounded border-gray-300 dark:border-zinc-600 
                                   text-gray-600 focus:ring-gray-500 dark:focus:ring-gray-400
                                   bg-white dark:bg-zinc-900"
                        />
                        <span class="text-sm text-gray-600 dark:text-gray-400">Remember me</span>
                    </label>
                </div>
            </div>

            <!-- Submit Button -->
            <button
                type="submit"
                class="mt-8 w-full flex items-center justify-center gap-2 px-5 py-3
                       bg-gray-900 dark:bg-gray-100
                       text-gray-100 dark:text-gray-900 font-semibold text-sm rounded-lg
                       transition-all duration-200
                       {isLoading || !username || !password
                         ? 'opacity-40 cursor-not-allowed'
                         : 'hover:bg-gray-800 dark:hover:bg-gray-200 active:scale-[0.98]'}"
                disabled={isLoading || !username || !password}
            >
                {#if isLoading}
                    <Icon icon="carbon:circle-dash" class="w-5 h-5 animate-spin" />
                    <span>Signing in...</span>
                {:else}
                    <Icon icon="carbon:login" class="w-5 h-5" />
                    <span>Sign in</span>
                {/if}
            </button>
        </div>

        <!-- Security Notice -->
        <div class="mt-6 text-center">
            <p class="text-xs text-gray-500 dark:text-gray-400 flex items-center justify-center gap-1.5">
                <Icon icon="carbon:locked" class="w-3.5 h-3.5" />
                <span>Secured with 256-bit encryption</span>
            </p>
        </div>
    </form>
</div>

<style>
    /* Custom checkbox styling */
    input[type="checkbox"] {
        appearance: none;
        -webkit-appearance: none;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 1rem;
        height: 1rem;
        border-radius: 0.25rem;
        transition: all 0.15s;
    }
    
    input[type="checkbox"]:checked {
        background-color: rgb(75 85 99);
        border-color: rgb(75 85 99);
    }
    
    input[type="checkbox"]:checked::before {
        content: "✓";
        color: white;
        font-size: 0.75rem;
        font-weight: bold;
    }

    :global(.dark) input[type="checkbox"]:checked {
        background-color: rgb(156 163 175);
        border-color: rgb(156 163 175);
    }
</style>