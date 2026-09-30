<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import Icon from "./Icon.vue";

// v-model is a "YYYY-MM-DD" string ("" when empty), same as a native date input.
const props = defineProps<{ placeholder?: string; min?: string; max?: string; align?: "left" | "right" }>();
const model = defineModel<string>({ required: true });

const open = ref(false);
const root = ref<HTMLElement | null>(null);
const view = ref(new Date()); // any day in the month being shown

const pad = (n: number) => String(n).padStart(2, "0");
const toKey = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const fromKey = (k: string) => {
  const [y, m, d] = k.split("-").map(Number);
  return new Date(y, m - 1, d);
};

const todayKey = toKey(new Date());
const weekdays = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

const display = computed(() =>
  model.value
    ? fromKey(model.value).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "2-digit" })
    : ""
);

const monthLabel = computed(() =>
  view.value.toLocaleDateString(undefined, { month: "long", year: "numeric" })
);

const days = computed(() => {
  const y = view.value.getFullYear();
  const m = view.value.getMonth();
  const first = new Date(y, m, 1);
  const offset = (first.getDay() + 6) % 7; // Monday-first
  const start = new Date(y, m, 1 - offset);
  return Array.from({ length: 42 }, (_, i) => {
    const d = new Date(start.getFullYear(), start.getMonth(), start.getDate() + i);
    const key = toKey(d);
    return {
      key,
      day: d.getDate(),
      inMonth: d.getMonth() === m,
      disabled: (!!props.min && key < props.min) || (!!props.max && key > props.max),
    };
  });
});

function toggle() {
  open.value = !open.value;
  if (open.value) view.value = model.value ? fromKey(model.value) : new Date();
}

function shift(months: number) {
  view.value = new Date(view.value.getFullYear(), view.value.getMonth() + months, 1);
}

function pick(key: string) {
  model.value = key;
  open.value = false;
}

function onDocDown(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false;
}
onMounted(() => document.addEventListener("mousedown", onDocDown));
onUnmounted(() => document.removeEventListener("mousedown", onDocDown));
</script>

<template>
  <div ref="root" class="relative" @keydown.esc="open = false">
    <button
      type="button"
      class="input input-sm h-8 flex items-center gap-2 text-left cursor-pointer"
      :class="{ 'border-accent ring-2 ring-accent/20': open }"
      @click="toggle"
    >
      <Icon name="calendar" :size="14" class="shrink-0 text-fg3" />
      <span class="flex-1 truncate whitespace-nowrap" :class="display ? 'text-fg' : 'text-fg3'">
        {{ display || placeholder || "Pick a date" }}
      </span>
      <span
        v-if="model"
        role="button"
        title="Clear date"
        class="shrink-0 -mr-1 rounded p-0.5 text-fg3 hover:text-fg hover:bg-surface2"
        @click.stop="model = ''"
      >
        <Icon name="x" :size="12" />
      </span>
    </button>

    <div
      v-if="open"
      class="absolute z-50 mt-1 w-64 rounded-lg border border-line bg-surface p-3 shadow-pop"
      :class="align === 'right' ? 'right-0' : 'left-0'"
    >
      <div class="flex items-center justify-between mb-2">
        <button type="button" class="btn-icon p-1" title="Previous month" @click="shift(-1)">
          <Icon name="chevron-left" :size="14" />
        </button>
        <span class="text-[13px] font-semibold">{{ monthLabel }}</span>
        <button type="button" class="btn-icon p-1" title="Next month" @click="shift(1)">
          <Icon name="chevron-right" :size="14" />
        </button>
      </div>

      <div class="grid grid-cols-7 gap-0.5 text-center">
        <span v-for="w in weekdays" :key="w" class="py-1 text-[11px] font-medium text-fg3">{{ w }}</span>
        <button
          v-for="d in days"
          :key="d.key"
          type="button"
          :disabled="d.disabled"
          class="num h-8 rounded-md text-[12px] transition-colors disabled:opacity-30 disabled:pointer-events-none"
          :class="[
            d.key === model
              ? 'bg-accent text-accent-fg font-semibold'
              : d.inMonth
                ? 'text-fg hover:bg-surface2'
                : 'text-fg3 hover:bg-surface2',
            d.key === todayKey && d.key !== model && 'ring-1 ring-inset ring-accent/60',
          ]"
          @click="pick(d.key)"
        >
          {{ d.day }}
        </button>
      </div>

      <div class="flex justify-between mt-2 pt-2 border-t border-line/60">
        <button type="button" class="btn btn-ghost btn-sm" @click="model = ''; open = false">Clear</button>
        <button type="button" class="btn btn-ghost btn-sm text-accent" @click="pick(todayKey)">Today</button>
      </div>
    </div>
  </div>
</template>
