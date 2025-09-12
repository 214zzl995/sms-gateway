<script>
    import { fly, fade } from "svelte/transition";
    import { apiClient } from "../../js/api";
    import { simCards } from "../../stores/simcards";

    // 组件状态
    let { value = $bindable() } = $props();
    let selectedSim = $state("");
    let recipient = $state("");
    let message = $state("");
    let isLoading = $state(false);
    let error = $state("");

    const self = (
        /** @type {MouseEvent & { currentTarget: EventTarget & HTMLDivElement; }} */ event,
        /** @type {{ (): void; call?: any; }} */ fn,
    ) => {
        if (event.target === event.currentTarget) {
            fn.call();
        }
    };

    // 关闭对话框
    const close = () => {
        value = false;
        resetForm();
    };

    // 提交表单
    const handleSubmit = async () => {
        if (!validateForm()) return;

        isLoading = true;
        try {
            await apiClient.sendSms(selectedSim, recipient, message);
            close();
        } catch (err) {
            error = err.message || "发送失败，请稍后重试";
        } finally {
            isLoading = false;
        }
    };

    // 表单验证
    const validateForm = () => {
        error = "";
        if (!selectedSim) error = "请选择SIM卡";
        else if (!/^\d{7,15}$/.test(recipient)) error = "请输入有效的电话号码";
        else if (!message.trim()) error = "请输入消息内容";
        return !error;
    };

    // 重置表单
    const resetForm = () => {
        selectedSim = "";
        recipient = "";
        message = "";
        error = "";
    };
</script>

{#if value}
    <!-- 遮罩层 -->
    <div
        transition:fade={{ duration: 150 }}
        class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/50"
        role="presentation"
        onclick={(event) => self(event, close)}
    >
        <!-- 对话框主体 -->
        <div
            transition:fly={{ y: -50, duration: 300 }}
            class="bg-white dark:bg-zinc-900 p-8 rounded-lg w-[90%] max-w-[500px] shadow-lg relative"
        >
            <h2
                class="mb-6 text-2xl font-semibold text-gray-800 dark:text-gray-100"
            >
                发送新消息
            </h2>

            <!-- 错误提示 -->
            {#if error}
                <div class="mb-4 p-3 rounded bg-red-100 text-red-700 text-sm">
                    {error}
                </div>
            {/if}

            <!-- 表单内容 -->
            <div class="mb-5">
                <label
                    class="block mb-2 text-gray-700 dark:text-gray-300 font-medium"
                >
                    选择设备:
                    <select
                        class="block w-full mt-2 px-3 py-2 rounded border border-gray-300 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition"
                        bind:value={selectedSim}
                    >
                        <option value="" disabled>请选择SIM卡</option>
                        {#each $simCards as simCard (simCard.id)}
                            <option value={simCard.id}>{simCard.alias || simCard.phone_number || simCard.id.slice(-8)}</option>
                        {/each}
                    </select>
                </label>
            </div>

            <div class="mb-5">
                <label
                    class="block mb-2 text-gray-700 dark:text-gray-300 font-medium"
                >
                    接收号码:
                    <input
                        type="tel"
                        bind:value={recipient}
                        placeholder="请输入电话号码"
                        class="block w-full mt-2 px-3 py-2 rounded border border-gray-300 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition"
                    />
                </label>
            </div>

            <div class="mb-4">
                <label
                    class="block mb-2 text-gray-700 dark:text-gray-300 font-medium"
                >
                    消息内容:
                    <textarea
                        bind:value={message}
                        placeholder="请输入消息内容（最多500字）"
                        maxlength="500"
                        class="block w-full mt-2 px-3 py-2 rounded border border-gray-300 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition resize-y min-h-[120px]"
                    ></textarea>
                </label>
            </div>

            <!-- 操作按钮 -->
            <div class="flex gap-3 mt-6">
                <button
                    type="button"
                    onclick={close}
                    disabled={isLoading}
                    class="flex-1 px-4 py-2 rounded bg-gray-100 dark:bg-zinc-800 text-gray-700 dark:text-gray-300 font-medium hover:bg-gray-200 dark:hover:bg-zinc-700 transition disabled:opacity-70 disabled:cursor-not-allowed"
                    >取消</button
                >
                <button
                    type="button"
                    onclick={handleSubmit}
                    disabled={isLoading}
                    class="flex-1 px-4 py-2 rounded bg-blue-600 text-white font-medium hover:bg-blue-700 transition disabled:opacity-70 disabled:cursor-not-allowed"
                    >{isLoading ? "发送中..." : "发送消息"}</button
                >
            </div>
        </div>
    </div>
{/if}
