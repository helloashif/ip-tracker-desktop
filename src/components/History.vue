<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import PageHeader from "./ui/PageHeader.vue";
import EmptyState from "./ui/EmptyState.vue";
import LabelEditor from "./ui/LabelEditor.vue";
import Icon from "./ui/Icon.vue";
import Select from "./ui/Select.vue";
import DatePicker from "./ui/DatePicker.vue";
import { api, type IpEvent, type HistoryFilter } from "../lib/api";
import { fmtDateTime, kindLabel, kindClass } from "../lib/format";
import { toast } from "../lib/toast";
import { confirm } from "../lib/confirm";

const props = defineProps<{ eventSeq: number }>();

const rows = ref<IpEvent[]>([]);
const total = ref(0);
const page = ref(0);
const pageSize = 50;
const search = ref("");
const kind = ref("all");
const from = ref("");
const to = ref("");
const loading = ref(true);
const exporting = ref(false);
const expanded = ref<number | null>(null);

const kinds = [
  { value: "all", label: "All events" },
  { value: "change", label: "IP changes" },
  { value: "offline", label: "Offline events" },
  { value: "online", label: "Back online" },
  { value: "start", label: "App started" },
];

function filter(paged = true): HistoryFilter {
  return {
    search: search.value.trim() || null,
    kind: kind.value,
    from: from.value ? new Date(`${from.value}T00:00:00`).toISOString() : null,
    to: to.value ? new Date(new Date(`${to.value}T00:00:00`).getTime() + 86_400_000).toISOString() : null,
    ...(paged ? { limit: pageSize, offset: page.value * pageSize } : {}),
  };
}

async function load() {
  loading.value = true;
  try {
    const f = filter();
    [rows.value, total.value] = await Promise.all([api.history(f), api.historyCount(f)]);
  } finally {
    loading.value = false;
  }
}

function hasFilters() {
  return !!(search.value || kind.value !== "all" || from.value || to.value);
}

function resetFilters() {
  search.value = "";
  kind.value = "all";
  from.value = "";
  to.value = "";
}

async function exportAs(format: "csv" | "json") {
  const stamp = new Date().toISOString().slice(0, 10);
  const path = await save({
    defaultPath: `ip-history-${stamp}.${format}`,
    filters: [{ name: format.toUpperCase(), extensions: [format] }],
  });
  if (!path) return;
  exporting.value = true;
  try {
    const n = format === "csv" ? await api.exportCsv(path, filter(false)) : await api.exportJson(path, filter(false));
    toast(`Exported ${n} event${n === 1 ? "" : "s"}`, "ok");
  } catch (e) {
    toast(`Export failed: ${e}`, "error");
  } finally {
    exporting.value = false;
  }
}

async function remove(e: IpEvent) {
  const ok = await confirm({
    title: "Delete this event?",
    body: `${kindLabel(e.kind)} at ${fmtDateTime(e.ts)} will be removed. Reports will no longer count it.`,
    confirmLabel: "Delete",
    danger: true,
  });
  if (!ok) return;
  await api.deleteEvent(e.id);
  toast("Event deleted", "ok");
  await load();
}

async function clearAll() {
  const ok = await confirm({
    title: "Delete all history?",
    body: "Every logged event will be removed. Labels are kept. This cannot be undone.",
    confirmLabel: "Delete everything",
    danger: true,
  });
  if (!ok) return;
  await api.clearHistory();
  page.value = 0;
  toast("History cleared", "ok");
  await load();
}

function onLabelSaved(ip: string, label: string | null) {
  rows.value.forEach((r) => {
    if (r.public_ip === ip) r.label = label;
  });
}

let debounce: number | undefined;
watch([search, kind, from, to], () => {
  page.value = 0;
  clearTimeout(debounce);
  debounce = window.setTimeout(load, 200);
});
watch(page, load);
watch(() => props.eventSeq, load);
onMounted(load);
</script>

<template>
  <div class="space-y-5">
    <PageHeader
      title="History"
      :subtitle="`${total} event${total === 1 ? '' : 's'} recorded in database`"
    >
      <div class="flex items-center gap-2">
        <button
          class="btn btn-secondary btn-sm h-9"
          :disabled="exporting || total === 0"
          title="Export as CSV"
          @click="exportAs('csv')"
        >
          <Icon name="download" :size="14" />
          <span>Export CSV</span>
        </button>
        <button
          class="btn btn-secondary btn-sm h-9"
          :disabled="exporting || total === 0"
          title="Export as JSON"
          @click="exportAs('json')"
        >
          <Icon name="download" :size="14" />
          <span>Export JSON</span>
        </button>
      </div>
    </PageHeader>

    <!-- Responsive Filter Bar -->
    <div class="card p-3.5 sm:p-4 flex flex-wrap items-center gap-3 bg-surface/90">
      <!-- Search Input with Icon -->
      <div class="relative flex-1 min-w-[220px]">
        <Icon name="search" :size="15" class="absolute left-3 top-1/2 -translate-y-1/2 text-fg3 pointer-events-none" />
        <input
          v-model="search"
          type="search"
          placeholder="Search by IP, provider, city or label…"
          class="input input-sm h-8 pl-9"
        />
      </div>

      <!-- Event Kind Filter -->
      <Select v-model="kind" :options="kinds" size="sm" class="w-40 shrink-0" />

      <!-- Date Range -->
      <div class="flex items-center gap-2 shrink-0">
        <DatePicker v-model="from" placeholder="From date" :max="to || undefined" class="w-40" />
        <span class="text-fg3 text-[12px] font-medium">to</span>
        <DatePicker v-model="to" placeholder="To date" :min="from || undefined" align="right" class="w-40" />
      </div>

      <!-- Reset Filters Button -->
      <button
        v-if="hasFilters()"
        class="btn btn-ghost btn-sm text-accent hover:text-accent font-medium h-8 whitespace-nowrap"
        @click="resetFilters"
      >
        <Icon name="x" :size="13" />
        <span>Reset filters</span>
      </button>
    </div>

    <!-- Loading Skeletons -->
    <div v-if="loading && rows.length === 0" class="card p-4 space-y-3">
      <div v-for="i in 8" :key="i" class="skeleton h-8"></div>
    </div>

    <!-- Empty State -->
    <EmptyState
      v-else-if="rows.length === 0"
      :title="hasFilters() ? 'No events match these filters' : 'No events logged yet'"
      :body="
        hasFilters()
          ? 'Try adjusting your search query or expanding the date range.'
          : 'Events will automatically appear here whenever your IP address changes or network goes offline.'
      "
    >
      <button v-if="hasFilters()" class="btn btn-secondary btn-sm" @click="resetFilters">
        Reset filters
      </button>
    </EmptyState>

    <!-- Native Data Table Card -->
    <div v-else class="card overflow-hidden shadow-sm flex flex-col">
      <div class="overflow-x-auto w-full">
        <table class="data">
          <thead>
            <tr>
              <th class="w-10"></th>
              <th>Timestamp</th>
              <th>Event</th>
              <th>Address</th>
              <th>Label</th>
              <th>Provider</th>
              <th>Location</th>
              <th class="w-12 text-right"></th>
            </tr>
          </thead>
          <tbody>
            <template v-for="e in rows" :key="e.id">
              <tr
                class="cursor-pointer group hover:bg-surface2/60 transition-colors"
                :class="{ 'bg-surface2/30': expanded === e.id }"
                @click="expanded = expanded === e.id ? null : e.id"
              >
                <!-- Expand Chevron -->
                <td class="text-fg3 group-hover:text-fg pl-3 pr-0">
                  <Icon
                    :name="expanded === e.id ? 'chevron-down' : 'chevron-right'"
                    :size="14"
                    class="transition-transform"
                  />
                </td>

                <!-- Timestamp -->
                <td class="num text-fg2 whitespace-nowrap text-[12px]">
                  {{ fmtDateTime(e.ts) }}
                </td>

                <!-- Event Kind Chip -->
                <td class="whitespace-nowrap">
                  <span class="chip" :class="kindClass(e.kind)">
                    {{ kindLabel(e.kind) }}
                  </span>
                </td>

                <!-- Address & Change arrow -->
                <td class="num font-medium text-fg whitespace-nowrap">
                  <template v-if="e.kind === 'change' && e.prev_public_ip">
                    <span class="text-fg3 font-normal">{{ e.prev_public_ip }}</span>
                    <span class="text-accent mx-1.5 font-sans">→</span>
                    <span class="font-bold">{{ e.public_ip }}</span>
                  </template>
                  <template v-else>
                    {{ e.public_ip ?? e.prev_public_ip ?? '—' }}
                  </template>
                </td>

                <!-- Label -->
                <td class="whitespace-nowrap" @click.stop>
                  <LabelEditor
                    v-if="e.public_ip"
                    :ip="e.public_ip"
                    :label="e.label"
                    compact
                    @saved="onLabelSaved(e.public_ip!, $event)"
                  />
                  <span v-else class="text-fg3">—</span>
                </td>

                <!-- Provider -->
                <td class="text-fg2 truncate max-w-[200px]" :title="e.isp ?? undefined">
                  {{ e.isp ?? '—' }}
                </td>

                <!-- Location -->
                <td class="text-fg2 whitespace-nowrap">
                  {{ [e.city, e.country].filter(Boolean).join(", ") || '—' }}
                </td>

                <!-- Actions -->
                <td class="text-right pr-3" @click.stop>
                  <button
                    class="btn-icon p-1.5 text-fg3 hover:text-danger hover:bg-danger/10 rounded transition-colors"
                    title="Delete event"
                    @click="remove(e)"
                  >
                    <Icon name="trash" :size="14" />
                  </button>
                </td>
              </tr>

              <!-- Expanded Details Drawer -->
              <tr v-if="expanded === e.id">
                <td colspan="8" class="!bg-surface2/60 p-4 border-b border-line">
                  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 text-[12px] bg-surface p-3.5 rounded-lg border border-line/60">
                    <div>
                      <span class="text-fg3 font-medium uppercase tracking-wider block mb-1">IPv6 Address</span>
                      <span class="num text-fg font-medium break-all">{{ e.public_ipv6 ?? 'Not detected' }}</span>
                    </div>

                    <div>
                      <span class="text-fg3 font-medium uppercase tracking-wider block mb-1">Previous IPv6</span>
                      <span class="num text-fg font-medium break-all">{{ e.prev_public_ipv6 ?? 'None' }}</span>
                    </div>

                    <div>
                      <span class="text-fg3 font-medium uppercase tracking-wider block mb-1">Local Network Interfaces</span>
                      <span class="num text-fg font-medium">{{ e.local_ips?.length ? e.local_ips.join(", ") : 'None' }}</span>
                    </div>

                    <div>
                      <span class="text-fg3 font-medium uppercase tracking-wider block mb-1">Exact UTC Timestamp</span>
                      <span class="num text-fg font-medium">{{ e.ts }}</span>
                    </div>
                  </div>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>

      <!-- Responsive Desktop Pagination & Clear Footer -->
      <div class="flex flex-wrap items-center justify-between gap-3 px-4 py-3 border-t border-line bg-surface text-fg2 text-[13px]">
        <button
          class="btn btn-ghost btn-sm text-danger hover:text-danger hover:bg-danger/10 font-medium"
          @click="clearAll"
        >
          <Icon name="trash" :size="13" />
          <span>Clear all history</span>
        </button>

        <div class="flex items-center gap-3 ml-auto">
          <span class="num text-[12px] text-fg3">
            Showing {{ page * pageSize + 1 }}–{{ Math.min((page + 1) * pageSize, total) }} of {{ total }}
          </span>

          <div v-if="total > pageSize" class="flex items-center gap-1.5">
            <button
              class="btn btn-secondary btn-sm h-8 px-2.5"
              :disabled="page === 0"
              @click="page--"
            >
              Previous
            </button>
            <span class="num text-[12px] px-2 font-medium">
              {{ page + 1 }} / {{ Math.ceil(total / pageSize) }}
            </span>
            <button
              class="btn btn-secondary btn-sm h-8 px-2.5"
              :disabled="(page + 1) * pageSize >= total"
              @click="page++"
            >
              Next
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
