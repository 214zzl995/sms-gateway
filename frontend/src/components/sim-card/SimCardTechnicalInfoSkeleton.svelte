<!-- frontend/src/lib/components/simcard/SimCardTechnicalInfoSkeleton.svelte -->
<div class="space-y-4">
    <h4 class="text-md font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-600 pb-2">
        Technical Details
    </h4>
    
    <!-- Signal Strength Skeleton -->
    <div class="flex items-center">
        <div class="w-5 h-5 mr-3 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
        <div class="flex-grow">
            <div class="h-3 bg-gray-300 dark:bg-gray-600 rounded w-24 mb-1 animate-pulse"></div>
            <div class="flex items-center">
                <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-16 mr-2 animate-pulse"></div>
                <div class="flex space-x-1">
                    <div class="w-1 h-2 bg-gray-300 dark:bg-gray-600 rounded-sm animate-pulse"></div>
                    <div class="w-1 h-2 bg-gray-300 dark:bg-gray-600 rounded-sm animate-pulse"></div>
                    <div class="w-1 h-2 bg-gray-300 dark:bg-gray-600 rounded-sm animate-pulse"></div>
                    <div class="w-1 h-2 bg-gray-300 dark:bg-gray-600 rounded-sm animate-pulse"></div>
                    <div class="w-1 h-2 bg-gray-300 dark:bg-gray-600 rounded-sm animate-pulse"></div>
                </div>
            </div>
        </div>
    </div>
    
    <!-- Enhanced Information Section -->
    <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-3">
        <h5 class="text-sm font-medium text-gray-600 dark:text-gray-400">Enhanced Information</h5>
        
        <!-- SMS Center Skeleton -->
        <div class="flex items-center text-sm">
            <div class="w-4 h-4 mr-2 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-20 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-32 ml-2 animate-pulse"></div>
        </div>
        
        <!-- SIM Status Skeleton -->
        <div class="flex items-center text-sm">
            <div class="w-4 h-4 mr-2 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-20 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-24 ml-2 animate-pulse"></div>
        </div>
        
        <!-- Memory Status Skeleton -->
        <div class="flex items-center text-sm">
            <div class="w-4 h-4 mr-2 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-16 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-28 ml-2 animate-pulse"></div>
        </div>
        
        <!-- Port Skeleton -->
        <div class="flex items-center text-sm">
            <div class="w-4 h-4 mr-2 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-8 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-20 ml-2 animate-pulse"></div>
        </div>
        
        <!-- Signal Detail Skeleton -->
        <div class="flex items-center text-sm">
            <div class="w-4 h-4 mr-2 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-24 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-36 ml-2 animate-pulse"></div>
        </div>
        
        <!-- Operator ID Skeleton -->
        <div class="flex items-center text-sm">
            <div class="w-4 h-4 mr-2 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-20 animate-pulse"></div>
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-16 ml-2 animate-pulse"></div>
        </div>
    </div>
</div>