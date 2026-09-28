<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Dashboard from "./components/Dashboard.vue";
import History from "./components/History.vue";
import Reports from "./components/Reports.vue";
import SettingsView from "./components/Settings.vue";
import Toasts from "./components/ui/Toasts.vue";
import ConfirmDialog from "./components/ui/ConfirmDialog.vue";
import { api, type CurrentStatus } from "./lib/api";
import { setTheme } from "./lib/theme";
import { toast } from "./lib/toast";
import { kindLabel } from "./lib/format";

type Tab = "dashboard" | "history" | "reports" | "settings";
const tab = ref<Tab>("dashboard");
const status = ref<CurrentStatus | null>(null);
const eventSeq = ref(0);

const tabs: { id: Tab; label: string; key: string }[] = [
  { id: "dashboard", label: "Overview", key: "1" },
  { id: "history", label: "History", key: "2" },
  { id: "reports", label: "Reports", key: "3" },
  { id: "settings", label: "Settings", key: "4" },
];

const unlisten: (() => void)[] = [];

function onKey(e: KeyboardEvent) {
  if (!(e.metaKey || e.ctrlKey)) return;
  const t = tabs.find((t) => t.key === e.key);
  if (t) { tab.value = t.id; e.preventDefault(); }
  if (e.key === "r") { api.checkNow(); e.preventDefault(); }
}

onMounted(async () => {
  const s = await api.settings();
  setTheme(s.theme);
  status.value = await api.current();
  unlisten.push(await api.onChecked((s) => { status.value = s; }));
  unlisten.push(await api.onEvent((e) => {
    eventSeq.value++;
    if (e.kind === "change") toast(`${kindLabel(e.kind)}: ${e.prev_public_ip ?? "?"} → ${e.public_ip}`, "info", 5000);
  }));
  unlisten.push(await api.onCopyIp(async (ip) => {
    await navigator.clipboard.writeText(ip);
    toast(`Copied ${ip}`, "ok");
  }));
  window.addEventListener("keydown", onKey);
});

onUnmounted(() => {
  unlisten.forEach((u) => u());
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <div class="flex h-full">
    <nav class="w-52 shrink-0 border-r border-line bg-surface flex flex-col">
      <div class="px-5 pt-5 pb-4 flex items-center gap-2.5">
        <span class="relative inline-flex h-2.5 w-2.5">
          <span
            v-if="status?.checking"
            class="absolute inline-flex h-full w-full rounded-full bg-accent opacity-60 animate-ping"
          ></span>
          <span
            class="relative inline-flex h-2.5 w-2.5 rounded-full"
            :class="status?.paused ? 'bg-fg3' : status?.online ? 'bg-ok' : 'bg-danger'"
          ></span>
        </span>
        <span class="font-semibold tracking-tight">IP Tracker</span>
      </div>

      <ul class="px-2.5 space-y-0.5">
        <li v-for="t in tabs" :key="t.id">
          <button
            class="w-full flex items-center justify-between px-3 py-2 rounded-md transition-colors"
            :class="tab === t.id ? 'bg-surface2 text-fg font-medium' : 'text-fg2 hover:bg-surface2/60 hover:text-fg'"
            @click="tab = t.id"
          >
            <span>{{ t.label }}</span>
            <kbd class="text-[11px] text-fg3 num">⌘{{ t.key }}</kbd>
          </button>
        </li>
      </ul>

      <div class="mt-auto px-5 py-4 border-t border-line">
        <p class="text-fg3 text-[12px]">Current address</p>
        <p class="num font-medium selectable truncate">{{ status?.public_ip ?? (status?.paused ? 'Paused' : 'Offline') }}</p>
      </div>
    </nav>

    <main class="flex-1 min-w-0 overflow-y-auto">
      <div class="p-8 max-w-5xl">
        <Dashboard v-if="tab === 'dashboard'" :status="status" :event-seq="eventSeq" />
        <History v-else-if="tab === 'history'" :event-seq="eventSeq" />
        <Reports v-else-if="tab === 'reports'" :event-seq="eventSeq" />
        <SettingsView v-else />
      </div>
    </main>

    <Toasts />
    <ConfirmDialog />
  </div>
</template>
