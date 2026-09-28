<script setup lang="ts">
defineProps<{ title: string; body?: string }>();
</script>

<template>
  <div class="card px-6 py-12 text-center">
    <p class="font-medium">{{ title }}</p>
    <p v-if="body" class="text-fg2 mt-1 max-w-sm mx-auto">{{ body }}</p>
    <div class="mt-4"><slot /></div>
  </div>
</template>
