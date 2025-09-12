<!-- frontend/src/lib/components/simcard/SimCardBasicInfoSkeleton.svelte -->
<div class="space-y-4">
    <h4 class="text-md font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-600 pb-2">
        Basic Information
    </h4>
    
    <!-- Phone Number Field Skeleton (EditableField format) -->
    <div class="flex items-center justify-between">
        <div class="flex items-center">
            <div class="w-5 h-5 mr-3 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div>
                <div class="h-3 bg-gray-300 dark:bg-gray-600 rounded w-20 mb-1 animate-pulse"></div>
                <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-32 animate-pulse"></div>
            </div>
        </div>
        <div class="flex items-center space-x-1">
            <div class="w-4 h-4 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
        </div>
    </div>
    
    <!-- Alias Field Skeleton (EditableField format) -->
    <div class="flex items-center justify-between">
        <div class="flex items-center">
            <div class="w-5 h-5 mr-3 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div>
                <div class="h-3 bg-gray-300 dark:bg-gray-600 rounded w-12 mb-1 animate-pulse"></div>
                <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-24 animate-pulse"></div>
            </div>
        </div>
        <div class="flex items-center space-x-1">
            <div class="w-4 h-4 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
        </div>
    </div>
    
    <!-- Network Status Field Skeleton (simple info field format) -->
    <div class="flex items-center">
        <div class="w-5 h-5 mr-3 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
        <div>
            <div class="h-3 bg-gray-300 dark:bg-gray-600 rounded w-24 mb-1 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-28 animate-pulse"></div>
        </div>
    </div>
    
    <!-- Operator Field Skeleton (simple info field format) -->
    <div class="flex items-center">
        <div class="w-5 h-5 mr-3 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
        <div>
            <div class="h-3 bg-gray-300 dark:bg-gray-600 rounded w-16 mb-1 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-20 animate-pulse"></div>
        </div>
    </div>
</div>