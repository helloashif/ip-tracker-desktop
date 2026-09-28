<script setup lang="ts">
import { confirmState, settle } from "../../lib/confirm";
</script>

<template>
  <div
    v-if="confirmState.open"
    class="fixed inset-0 z-40 flex items-center justify-center bg-black/40 p-6"
    @click.self="settle(false)"
    @keydown.esc="settle(false)"
  >
    <div class="card w-full max-w-md p-5 shadow-pop" role="dialog" aria-modal="true">
      <h2 class="text-base font-semibold mb-1.5">{{ confirmState.title }}</h2>
      <p class="text-fg2 mb-5">{{ confirmState.body }}</p>
      <div class="flex justify-end gap-2">
        <button class="btn btn-secondary" @click="settle(false)">Cancel</button>
        <button class="btn" :class="confirmState.danger ? 'bg-danger text-white hover:brightness-95' : 'btn-primary'" autofocus @click="settle(true)">
          {{ confirmState.confirmLabel }}
        </button>
      </div>
    </div>
  </div>
</template>
