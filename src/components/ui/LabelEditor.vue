<script setup lang="ts">
import { ref, nextTick } from "vue";
import { api } from "../../lib/api";
import { toast } from "../../lib/toast";

const props = defineProps<{ ip: string; label: string | null; compact?: boolean }>();
const emit = defineEmits<{ (e: "saved", label: string | null): void }>();

const editing = ref(false);
const draft = ref("");
const input = ref<HTMLInputElement | null>(null);

async function start() {
  draft.value = props.label ?? "";
  editing.value = true;
  await nextTick();
  input.value?.focus();
  input.value?.select();
}

async function save() {
  if (!editing.value) return;
  editing.value = false;
  const next = draft.value.trim();
  if (next === (props.label ?? "")) return;
  try {
    await api.setLabel(props.ip, next);
    emit("saved", next || null);
    toast(next ? `Labeled ${props.ip} as “${next}”` : "Label removed", "ok");
  } catch (e) {
    toast(`Couldn't save label: ${e}`, "error");
  }
}
</script>

<template>
  <span class="inline-flex items-center">
    <input
      v-if="editing"
      ref="input"
      v-model="draft"
      class="input input-sm w-36"
      placeholder="e.g. Home, Office VPN"
      maxlength="40"
      @keydown.enter.prevent="save"
      @keydown.esc="editing = false"
      @blur="save"
    />
    <button
      v-else
      class="chip transition-colors"
      :class="label ? 'bg-accent/15 text-accent hover:bg-accent/25' : 'text-fg3 hover:text-fg2 hover:bg-surface2'"
      :title="label ? 'Edit label' : 'Add a label to this address'"
      @click="start"
    >
      {{ label ?? (compact ? '+ label' : 'Add label') }}
    </button>
  </span>
</template>
