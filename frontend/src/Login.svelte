<script>
    import { updateStorageValue } from "./js/storage";
    import { fade } from 'svelte/transition'; 

    let username = "";
    let password = "";
    let error = "";
    let isLoading = false;

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
                    error = "Invalid credentials (Unauthorized)";
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
</script>

<div class="fixed top-4 left-4 sm:top-6 sm:left-6 flex items-center space-x-2 z-10">
    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
        </svg>
    <span class="text-sm font-medium text-gray-600 dark:text-gray-400">SMS Geteway</span>
</div>

<div
    class="min-h-screen w-screen flex items-center justify-center bg-gray-50 dark:bg-[#0f1116] transition-colors duration-300 px-4 sm:px-6 lg:px-8"
>
    <form
        class="w-full max-w-xs sm:max-w-sm mx-auto rounded-2xl bg-white dark:bg-[#202230] border border-gray-200 dark:border-neutral-800 px-6 sm:px-8 py-10 flex flex-col gap-6"
        on:submit|preventDefault={handleSubmit}
        autocomplete="off"
        in:fade={{ duration: 200 }} >
        <h2
            class="text-2xl md:text-3xl text-center font-bold text-gray-900 dark:text-white mb-4 tracking-tight"
        >
            Sign in
        </h2>

        {#if error}
            <div
                class="bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 border border-red-300 dark:border-red-700/50 rounded-lg py-2 px-4 text-center text-xs sm:text-sm font-semibold select-none"
                role="alert"
            >
                {error}
            </div>
        {/if}

        <div class="flex flex-col gap-1">
            <label
                for="username"
                class="text-xs sm:text-sm font-semibold text-gray-500 dark:text-gray-300"
                >Username:</label
            >
            <input
                id="username"
                type="text"
                autocomplete="username"
                bind:value={username}
                placeholder=""
                class="peer block w-full border-b border-gray-300 dark:border-gray-600 bg-transparent text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 text-sm sm:text-base py-1.5 focus:border-green-500 focus:outline-none transition"
                disabled={isLoading}
                required
            />
        </div>

        <div class="flex flex-col gap-1">
            <label
                for="password"
                class="text-xs sm:text-sm font-semibold text-gray-500 dark:text-gray-300"
                >Password:</label
            >
            <input
                id="password"
                type="password"
                autocomplete="current-password"
                bind:value={password}
                placeholder=""
                class="peer block w-full border-b border-gray-300 dark:border-gray-600 bg-transparent text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 text-sm sm:text-base py-1.5 focus:border-green-500 focus:outline-none transition"
                disabled={isLoading}
                required
            />
        </div>

        {#if isLoading}
            <div
                class="text-center text-gray-600 dark:text-gray-400 py-3 select-none text-sm"
                aria-live="polite"
            >
                Authenticating...
            </div>
        {:else}
            <button
                type="submit"
                aria-label="Login button"
                class="mt-4 w-full rounded-xl bg-green-600 hover:bg-green-700 active:bg-green-800 focus:outline-none focus:ring-4 focus:ring-green-400 focus:ring-opacity-50 text-white font-semibold text-base sm:text-lg py-2.5 transition select-none disabled:opacity-60 disabled:cursor-not-allowed"
                disabled={isLoading || !username || !password}
            >
                Sign In
            </button>
        {/if}
    </form>
</div>