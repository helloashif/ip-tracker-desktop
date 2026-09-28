<script setup lang="ts">
import { toasts } from "../../lib/toast";
</script>

<template>
  <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 items-end pointer-events-none" aria-live="polite">
    <div
      v-for="t in toasts" :key="t.id"
      class="pointer-events-auto card px-3.5 py-2.5 shadow-pop flex items-center gap-2 max-w-sm"
    >
      <span
        class="h-2 w-2 rounded-full shrink-0"
        :class="t.tone === 'ok' ? 'bg-ok' : t.tone === 'error' ? 'bg-danger' : 'bg-accent'"
      ></span>
      <span>{{ t.text }}</span>
    </div>
  </div>
</template>
