<script setup lang="ts" generic="T extends string | number">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import Icon from "./Icon.vue";

const props = defineProps<{
  options: { value: T; label: string }[];
  size?: "sm" | "md";
  id?: string;
}>();
const model = defineModel<T>({ required: true });

const open = ref(false);
const active = ref(0);
const root = ref<HTMLElement | null>(null);
const list = ref<HTMLElement | null>(null);

const current = computed(() => props.options.find((o) => o.value === model.value));

async function toggle() {
  open.value = !open.value;
  if (open.value) {
    active.value = Math.max(0, props.options.findIndex((o) => o.value === model.value));
    await nextTick();
    list.value?.children[active.value]?.scrollIntoView({ block: "nearest" });
  }
}

function pick(v: T) {
  model.value = v;
  open.value = false;
}

function onKey(e: KeyboardEvent) {
  if (!open.value) {
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
      e.preventDefault();
      toggle();
    }
    return;
  }
  if (e.key === "Escape" || e.key === "Tab") {
    open.value = false;
  } else if (e.key === "ArrowDown") {
    e.preventDefault();
    active.value = Math.min(props.options.length - 1, active.value + 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    active.value = Math.max(0, active.value - 1);
  } else if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    pick(props.options[active.value].value);
  }
  list.value?.children[active.value]?.scrollIntoView({ block: "nearest" });
}

function onDocDown(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false;
}
onMounted(() => document.addEventListener("mousedown", onDocDown));
onUnmounted(() => document.removeEventListener("mousedown", onDocDown));
</script>

<template>
  <div ref="root" class="relative">
    <button
      :id="id"
      type="button"
      class="input flex items-center justify-between gap-2 text-left cursor-pointer"
      :class="[size === 'sm' ? 'input-sm h-8' : 'h-9', open && 'border-accent ring-2 ring-accent/20']"
      aria-haspopup="listbox"
      :aria-expanded="open"
      @click="toggle"
      @keydown="onKey"
    >
      <span class="truncate">{{ current?.label ?? "—" }}</span>
      <Icon
        name="chevron-down"
        :size="14"
        class="shrink-0 text-fg3 transition-transform"
        :class="{ 'rotate-180': open }"
      />
    </button>

    <ul
      v-if="open"
      ref="list"
      role="listbox"
      class="absolute z-50 mt-1 w-full min-w-max max-h-64 overflow-auto rounded-lg border border-line bg-surface p-1 shadow-pop"
    >
      <li
        v-for="(o, i) in options"
        :key="String(o.value)"
        role="option"
        :aria-selected="o.value === model"
        class="flex items-center justify-between gap-3 rounded-md px-2.5 py-1.5 cursor-pointer whitespace-nowrap"
        :class="[
          size === 'sm' ? 'text-[12px]' : 'text-[13px]',
          i === active ? 'bg-surface2 text-fg' : 'text-fg2',
          o.value === model && 'text-accent font-medium',
        ]"
        @mouseenter="active = i"
        @mousedown.prevent="pick(o.value)"
      >
        <span>{{ o.label }}</span>
        <Icon v-if="o.value === model" name="check" :size="13" />
      </li>
    </ul>
  </div>
</template>
