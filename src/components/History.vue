<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import PageHeader from "./ui/PageHeader.vue";
import EmptyState from "./ui/EmptyState.vue";
import LabelEditor from "./ui/LabelEditor.vue";
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
  { id: "all", label: "All events" },
  { id: "change", label: "Changes" },
  { id: "offline", label: "Offline" },
  { id: "online", label: "Back online" },
  { id: "start", label: "Started" },
];

function filter(paged = true): HistoryFilter {
  return {
    search: search.value.trim() || null,
    kind: kind.value,
    from: from.value ? new Date(from.value).toISOString() : null,
    to: to.value ? new Date(new Date(to.value).getTime() + 86_400_000).toISOString() : null,
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
  search.value = ""; kind.value = "all"; from.value = ""; to.value = "";
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
  rows.value.forEach((r) => { if (r.public_ip === ip) r.label = label; });
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
  <PageHeader title="History" :subtitle="`${total} event${total === 1 ? '' : 's'} logged`">
    <button class="btn btn-secondary" :disabled="exporting || total === 0" @click="exportAs('csv')">Export CSV</button>
    <button class="btn btn-secondary" :disabled="exporting || total === 0" @click="exportAs('json')">Export JSON</button>
  </PageHeader>

  <div class="card p-4 mb-4 flex flex-wrap items-end gap-3">
    <label class="flex-1 min-w-48">
      <span class="field-label text-[13px]">Search</span>
      <input v-model="search" type="search" placeholder="IP, provider, city or label" class="input input-sm" />
    </label>
    <label>
      <span class="field-label text-[13px]">Event</span>
      <select v-model="kind" class="input input-sm w-40">
        <option v-for="k in kinds" :key="k.id" :value="k.id">{{ k.label }}</option>
      </select>
    </label>
    <label>
      <span class="field-label text-[13px]">From</span>
      <input v-model="from" type="date" class="input input-sm" />
    </label>
    <label>
      <span class="field-label text-[13px]">To</span>
      <input v-model="to" type="date" class="input input-sm" />
    </label>
    <button v-if="hasFilters()" class="btn btn-ghost btn-sm" @click="resetFilters">Reset</button>
  </div>

  <div v-if="loading && rows.length === 0" class="card p-4 space-y-3">
    <div v-for="i in 6" :key="i" class="skeleton h-6"></div>
  </div>

  <EmptyState
    v-else-if="rows.length === 0"
    :title="hasFilters() ? 'No events match these filters' : 'No events yet'"
    :body="hasFilters() ? 'Try a wider date range or a different search.' : 'Events appear here as your address changes or your connection drops.'"
  >
    <button v-if="hasFilters()" class="btn btn-secondary btn-sm" @click="resetFilters">Reset filters</button>
  </EmptyState>

  <div v-else class="card overflow-hidden">
    <div class="overflow-x-auto">
      <table class="data">
        <thead>
          <tr>
            <th>When</th>
            <th>Event</th>
            <th>Address</th>
            <th>Label</th>
            <th>Provider</th>
            <th>Location</th>
            <th class="w-10"></th>
          </tr>
        </thead>
        <tbody>
          <template v-for="e in rows" :key="e.id">
            <tr class="cursor-pointer" @click="expanded = expanded === e.id ? null : e.id">
              <td class="num text-fg2 whitespace-nowrap">{{ fmtDateTime(e.ts) }}</td>
              <td><span class="chip" :class="kindClass(e.kind)">{{ kindLabel(e.kind) }}</span></td>
              <td class="num selectable whitespace-nowrap">
                <span v-if="e.kind === 'change' && e.prev_public_ip" class="text-fg3">{{ e.prev_public_ip }} → </span>{{ e.public_ip ?? e.prev_public_ip ?? '—' }}
              </td>
              <td @click.stop>
                <LabelEditor v-if="e.public_ip" :ip="e.public_ip" :label="e.label" compact @saved="onLabelSaved(e.public_ip!, $event)" />
                <span v-else class="text-fg3">—</span>
              </td>
              <td class="text-fg2">{{ e.isp ?? '—' }}</td>
              <td class="text-fg2 whitespace-nowrap">{{ [e.city, e.country].filter(Boolean).join(", ") || '—' }}</td>
              <td @click.stop>
                <button class="btn btn-ghost btn-sm text-fg3 hover:text-danger" title="Delete event" @click="remove(e)">✕</button>
              </td>
            </tr>
            <tr v-if="expanded === e.id">
              <td colspan="7" class="!bg-surface2/50">
                <dl class="grid sm:grid-cols-3 gap-x-6 gap-y-2 text-[13px]">
                  <div>
                    <dt class="text-fg3">IPv6</dt>
                    <dd class="num selectable break-all">{{ e.public_ipv6 ?? '—' }}</dd>
                  </div>
                  <div>
                    <dt class="text-fg3">Previous IPv6</dt>
                    <dd class="num selectable break-all">{{ e.prev_public_ipv6 ?? '—' }}</dd>
                  </div>
                  <div>
                    <dt class="text-fg3">Local addresses</dt>
                    <dd class="num selectable">{{ e.local_ips.join(", ") || '—' }}</dd>
                  </div>
                  <div>
                    <dt class="text-fg3">Timestamp (UTC)</dt>
                    <dd class="num selectable">{{ e.ts }}</dd>
                  </div>
                </dl>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <div class="flex items-center justify-between gap-3 px-4 py-3 border-t border-line text-fg2">
      <button class="btn btn-ghost btn-sm text-danger" @click="clearAll">Clear all history</button>
      <div v-if="total > pageSize" class="flex items-center gap-3">
        <button class="btn btn-secondary btn-sm" :disabled="page === 0" @click="page--">Previous</button>
        <span class="num">Page {{ page + 1 }} of {{ Math.ceil(total / pageSize) }}</span>
        <button class="btn btn-secondary btn-sm" :disabled="(page + 1) * pageSize >= total" @click="page++">Next</button>
      </div>
    </div>
  </div>
</template>
