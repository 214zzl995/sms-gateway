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
    <div class="p-2.5 bg-gray-50 dark:bg-zinc-800 rounded-xl border border-gray-200 dark:border-zinc-700">
        <Icon icon="carbon:send-filled" class="w-5 h-5 text-gray-700 dark:text-gray-300" />
    </div>
    <div>
        <h1 class="text-lg font-semibold text-gray-900 dark:text-white">SMS Gateway</h1>
        <p class="text-xs text-gray-500 dark:text-gray-400">Secure messaging platform</p>
    </div>
</div>

<!-- Main Container -->
<div class="min-h-screen w-screen flex items-center justify-center bg-white dark:bg-zinc-900 transition-colors duration-300 px-4 sm:px-6 lg:px-8">
    <!-- Decorative background elements -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none">
        <div class="absolute -top-40 -right-40 w-80 h-80 bg-blue-500/5 dark:bg-blue-500/10 rounded-full blur-3xl"></div>
        <div class="absolute -bottom-40 -left-40 w-80 h-80 bg-green-500/5 dark:bg-green-500/10 rounded-full blur-3xl"></div>
    </div>

    <!-- Login Form -->
    <form
        class="relative w-full max-w-md mx-auto"
        onsubmit={handleSubmit}
        autocomplete="off"
        in:fly={{ y: 20, duration: 400, easing: quintOut }}
    >
        <!-- Card Container -->
        <div class="bg-gray-50/50 dark:bg-zinc-800/50 backdrop-blur-xl border border-gray-200 dark:border-zinc-700 rounded-2xl shadow-xl shadow-gray-900/5 dark:shadow-black/20 p-8 sm:p-10">
            <!-- Header -->
            <div class="text-center mb-8">
                <div class="inline-flex items-center justify-center w-16 h-16 bg-gradient-to-br from-blue-500 to-blue-600 rounded-2xl shadow-lg shadow-blue-500/25 mb-4">
                    <Icon icon="carbon:user-avatar-filled" class="w-8 h-8 text-white" />
                </div>
                <h2 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">
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
                                   border border-gray-200 dark:border-zinc-700 rounded-xl
                                   text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500
                                   transition-all duration-200
                                   focus:outline-none focus:border-gray-400 dark:focus:border-zinc-600
                                   hover:border-gray-300 dark:hover:border-zinc-600
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
                                   border border-gray-200 dark:border-zinc-700 rounded-xl
                                   text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500
                                   transition-all duration-200
                                   focus:outline-none focus:border-gray-400 dark:focus:border-zinc-600
                                   hover:border-gray-300 dark:hover:border-zinc-600
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

                <!-- Remember me & Forgot password -->
                <div class="flex items-center justify-between">
                    <label class="flex items-center gap-2 cursor-pointer">
                        <input 
                            type="checkbox" 
                            class="w-4 h-4 rounded border-gray-300 dark:border-zinc-600 
                                   text-blue-600 focus:ring-blue-500 dark:focus:ring-blue-400
                                   bg-white dark:bg-zinc-900"
                        />
                        <span class="text-sm text-gray-600 dark:text-gray-400">Remember me</span>
                    </label>
                    <a href="#forgot-password" class="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 transition-colors">
                        Forgot password?
                    </a>
                </div>
            </div>

            <!-- Submit Button -->
            <button
                type="submit"
                class="mt-8 w-full flex items-center justify-center gap-2 px-5 py-3
                       bg-gradient-to-r from-blue-500 to-blue-600
                       text-white font-medium text-sm rounded-xl
                       transition-all duration-200
                       {isLoading || !username || !password
                         ? 'opacity-40 cursor-not-allowed'
                         : 'hover:from-blue-600 hover:to-blue-700 active:from-blue-700 active:to-blue-800 transform hover:-translate-y-0.5 active:translate-y-0'}"
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

            <!-- Divider -->
            <div class="relative my-8">
                <div class="absolute inset-0 flex items-center">
                    <div class="w-full border-t border-gray-200 dark:border-zinc-700"></div>
                </div>
                <div class="relative flex justify-center text-sm">
                    <span class="px-4 bg-gray-50/50 dark:bg-zinc-800/50 text-gray-500 dark:text-gray-400">
                        Or continue with
                    </span>
                </div>
            </div>

            <!-- Social Login Options -->
            <div class="grid grid-cols-2 gap-3">
                <button
                    type="button"
                    class="flex items-center justify-center gap-2 px-4 py-2.5
                           bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700
                           text-gray-700 dark:text-gray-300 text-sm font-medium rounded-xl
                           hover:bg-gray-50 dark:hover:bg-zinc-800 hover:border-gray-300 dark:hover:border-zinc-600
                           transition-all duration-200"
                >
                    <Icon icon="carbon:logo-github" class="w-5 h-5" />
                    <span>GitHub</span>
                </button>
                <button
                    type="button"
                    class="flex items-center justify-center gap-2 px-4 py-2.5
                           bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700
                           text-gray-700 dark:text-gray-300 text-sm font-medium rounded-xl
                           hover:bg-gray-50 dark:hover:bg-zinc-800 hover:border-gray-300 dark:hover:border-zinc-600
                           transition-all duration-200"
                >
                    <Icon icon="carbon:logo-google" class="w-5 h-5" />
                    <span>Google</span>
                </button>
            </div>

            <!-- Sign up link -->
            <p class="mt-8 text-center text-sm text-gray-600 dark:text-gray-400">
                Don't have an account? 
                <a href="#sign-up" class="font-medium text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 transition-colors">
                    Sign up
                </a>
            </p>
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
        background-color: rgb(59 130 246);
        border-color: rgb(59 130 246);
    }
    
    input[type="checkbox"]:checked::before {
        content: "✓";
        color: white;
        font-size: 0.75rem;
        font-weight: bold;
    }

    :global(.dark) input[type="checkbox"]:checked {
        background-color: rgb(96 165 250);
        border-color: rgb(96 165 250);
    }
</style>