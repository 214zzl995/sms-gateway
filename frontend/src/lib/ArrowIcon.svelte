<script>
    let { isExpanded,onclick } = $props();
</script>

<svg
    class="arrow-icon {isExpanded ? 'expanded' : 'collapsed'}"
    width="18"
    height="18"
    {onclick}
    viewBox="0 0 24 24"
    aria-hidden="true"
>
    <path
        fill="currentColor"
        d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6 1.41-1.41z"
    />
</svg>

<style>
    .arrow-icon {
        transition: transform 0.3s ease;
        transform-origin: center;
    }
    .expanded {
        transform: rotate(0deg);
    }
    .collapsed {
        transform: rotate(-90deg);
    }
</style>
