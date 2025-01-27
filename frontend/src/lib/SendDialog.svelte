<script>
    import { fly, fade } from "svelte/transition";
    import { apiClient } from "../js/api";
    import { devices } from "../stores/devices";

    // 组件状态
    let { value = $bindable() } = $props();
    let selectedDevice = $state("");
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
            await apiClient.sendSms(selectedDevice, recipient, message);
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
        if (!selectedDevice) error = "请选择设备";
        else if (!/^\d{7,15}$/.test(recipient)) error = "请输入有效的电话号码";
        else if (!message.trim()) error = "请输入消息内容";
        return !error;
    };

    // 重置表单
    const resetForm = () => {
        selectedDevice = "";
        recipient = "";
        message = "";
        error = "";
    };
</script>

{#if value}
    <!-- 遮罩层 -->
    <div
        transition:fade={{ duration: 150 }}
        class="dialog-mask"
        role="presentation"
        onclick={(event) => {
            self(event, close);
        }}
    >
        <!-- 对话框主体 -->
        <div
            transition:fly={{ y: -50, duration: 300 }}
            class="dialog-container"
        >
            <h2>发送新消息</h2>

            <!-- 错误提示 -->
            {#if error}
                <div class="error-message">{error}</div>
            {/if}

            <!-- 表单内容 -->
            <div class="form-group">
                <label
                    >选择设备:

                    <select bind:value={selectedDevice}>
                        <option value="" disabled>请选择设备</option>
                        {#each $devices as device (device.name)}
                            <option value={device.name}>{device.name}</option>
                        {/each}
                    </select>
                </label>
            </div>

            <div class="form-group">
                <label
                    >接收号码: <input
                        type="tel"
                        bind:value={recipient}
                        placeholder="请输入电话号码"
                    /></label
                >
            </div>

            <div class="form-group">
                <label
                    >消息内容: <textarea
                        bind:value={message}
                        placeholder="请输入消息内容（最多500字）"
                        maxlength="500"
                    ></textarea></label
                >
            </div>

            <!-- 操作按钮 -->
            <div class="action-buttons">
                <button class="cancel" onclick={close} disabled={isLoading}
                    >取消</button
                >
                <button
                    class="submit"
                    onclick={handleSubmit}
                    disabled={isLoading}
                >
                    {isLoading ? "发送中..." : "发送消息"}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    .dialog-mask {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .dialog-container {
        background: white;
        padding: 2rem;
        border-radius: 8px;
        width: 90%;
        max-width: 500px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }

    h2 {
        margin: 0 0 1.5rem;
        color: #333;
        font-size: 1.5rem;
    }

    .form-group {
        margin-bottom: 1.2rem;
    }

    label {
        display: block;
        margin-bottom: 0.5rem;
        color: #666;
        font-weight: 500;
    }

    select,
    input,
    textarea {
        width: -webkit-fill-available;
        padding: 0.8rem;
        border: 1px solid #ddd;
        border-radius: 4px;
        font-size: 1rem;
    }

    textarea {
        height: 120px;
        resize: vertical;
    }

    .error-message {
        color: #dc3545;
        background: #ffe6e6;
        padding: 0.8rem;
        border-radius: 4px;
        margin-bottom: 1.2rem;
        font-size: 0.9rem;
    }

    .action-buttons {
        display: flex;
        gap: 1rem;
        margin-top: 1.5rem;
    }

    button {
        flex: 1;
        padding: 0.8rem 1.2rem;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 500;
        transition: all 0.2s;
    }

    .submit {
        background: #007bff;
        color: white;
    }

    .submit:hover:not(:disabled) {
        background: #0056b3;
    }

    .cancel {
        background: #f8f9fa;
        color: #666;
    }

    .cancel:hover:not(:disabled) {
        background: #e9ecef;
    }

    button:disabled {
        opacity: 0.7;
        cursor: not-allowed;
    }
</style>
