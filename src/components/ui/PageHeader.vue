<script setup lang="ts">
defineProps<{ title: string; subtitle?: string }>();
</script>

<template>
  <header class="flex flex-wrap items-center justify-between gap-4 mb-6">
    <div class="min-w-0">
      <h1 class="text-2xl sm:text-3xl font-bold tracking-tight text-fg">{{ title }}</h1>
      <p v-if="subtitle" class="text-fg2 mt-1 text-[13px] leading-relaxed max-w-2xl">{{ subtitle }}</p>
    </div>
    <div class="flex flex-wrap items-center gap-2.5 shrink-0"><slot /></div>
  </header>
</template>
