<script setup lang="ts">
defineProps<{ title: string; subtitle?: string }>();
</script>

<template>
  <header class="flex items-start justify-between gap-4 mb-6">
    <div>
      <h1 class="text-xl font-semibold tracking-tight">{{ title }}</h1>
      <p v-if="subtitle" class="text-fg2 mt-0.5">{{ subtitle }}</p>
    </div>
    <div class="flex items-center gap-2 shrink-0"><slot /></div>
  </header>
</template>
