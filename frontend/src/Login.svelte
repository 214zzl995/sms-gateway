<script>
    import { updateStorageValue } from "./js/storage";

    let username = "";
    let password = "";
    let error = "";
    let isLoading = false;

    const handleSubmit = (e) => {
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

        const response = await fetch('/api/check', {
            method: 'GET',
            headers: {
                'Authorization': `Basic ${authToken}`,
                'Content-Type': 'application/json'
            }
        });

        switch (response.status) {
            case 204: 
                await updateStorageValue("auth", {
                    username,
                    token: authToken 
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
        error = err.message.includes('Failed to fetch') 
            ? "Unable to connect to the server" 
            : "Authentication process failed";
    } finally {
        isLoading = false;
    }
};
</script>

<div class="login-wrapper">
    <form class="login-container" on:submit={handleSubmit}>
        <h2>System Login</h2>

        {#if error}
            <div class="error" role="alert">{error}</div>
        {/if}

        <div class="form-group">
            <label for="username">Username</label>
            <input
                id="username"
                type="text"
                autocomplete="username"
                bind:value={username}
                placeholder="Enter your username"
                disabled={isLoading}
                required
            />
        </div>

        <div class="form-group">
            <label for="password">Password</label>
            <input
                id="password"
                type="password"
                autocomplete="current-password"
                bind:value={password}
                placeholder="Enter your password"
                disabled={isLoading}
                required
            />
        </div>

        {#if isLoading}
            <div class="loading" aria-live="polite">Authenticating...</div>
        {:else}
            <button type="submit" aria-label="Login button"> Sign In </button>
        {/if}
    </form>
</div>

<style>
    .login-wrapper {
        display: flex;
        justify-content: center;
        align-items: center;
        min-height: 100vh;
        min-width: 100vw;
        background: #f0f2f5;
    }

    .login-container {
        width: 100%;
        max-width: 400px;
        padding: 2.5rem;
        background: #ffffff;
        border-radius: 12px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
        transition: transform 0.2s ease;
    }

    h2 {
        text-align: center;
        color: #1a1a1a;
        margin-bottom: 2rem;
        font-size: 1.8rem;
    }

    .form-group {
        margin-bottom: 1.8rem;
    }

    label {
        display: block;
        margin-bottom: 0.6rem;
        color: #4a5568;
        font-size: 0.9rem;
        font-weight: 500;
    }

    input {
        width: -webkit-fill-available;
        padding: 0.9rem 1.2rem;
        border: 1px solid #e2e8f0;
        border-radius: 8px;
        font-size: 1rem;
        transition: border-color 0.2s ease;
    }

    input:focus {
        outline: none;
        border-color: #4299e1;
        box-shadow: 0 0 0 3px rgba(66, 153, 225, 0.1);
    }

    button {
        width: 100%;
        padding: 1rem;
        background: #4299e1;
        color: white;
        border: none;
        border-radius: 8px;
        font-size: 1rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    button:hover {
        background: #3182ce;
        transform: translateY(-1px);
    }

    button:active {
        transform: translateY(0);
    }

    .error {
        color: #e53e3e;
        margin: 1.2rem 0;
        text-align: center;
        font-size: 0.9rem;
    }

    .loading {
        text-align: center;
        color: #718096;
        margin: 1.5rem 0;
        font-size: 0.95rem;
    }
</style>
