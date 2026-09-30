<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Dashboard from "./components/Dashboard.vue";
import History from "./components/History.vue";
import Reports from "./components/Reports.vue";
import SettingsView from "./components/Settings.vue";
import Toasts from "./components/ui/Toasts.vue";
import ConfirmDialog from "./components/ui/ConfirmDialog.vue";
import Icon from "./components/ui/Icon.vue";
import { api, type CurrentStatus } from "./lib/api";
import { setTheme } from "./lib/theme";
import { toast } from "./lib/toast";
import { kindLabel } from "./lib/format";

type Tab = "dashboard" | "history" | "reports" | "settings";
const tab = ref<Tab>("dashboard");
const status = ref<CurrentStatus | null>(null);
const eventSeq = ref(0);
const sidebarCollapsed = ref(false);
const isFullscreen = ref(false);
const checking = ref(false);

const appWindow = getCurrentWindow();

const tabs: { id: Tab; label: string; key: string; icon: "overview" | "history" | "reports" | "settings" }[] = [
  { id: "dashboard", label: "Overview", key: "1", icon: "overview" },
  { id: "history", label: "History", key: "2", icon: "history" },
  { id: "reports", label: "Reports", key: "3", icon: "reports" },
  { id: "settings", label: "Settings", key: "4", icon: "settings" },
];

const unlisten: (() => void)[] = [];

function onKey(e: KeyboardEvent) {
  if (e.key === "F11") {
    toggleFullscreen();
    e.preventDefault();
    return;
  }
  if (!(e.metaKey || e.ctrlKey)) return;
  const t = tabs.find((t) => t.key === e.key);
  if (t) {
    tab.value = t.id;
    e.preventDefault();
  }
  if (e.key === "r") {
    triggerCheck();
    e.preventDefault();
  }
  if (e.key === "b") {
    sidebarCollapsed.value = !sidebarCollapsed.value;
    e.preventDefault();
  }
}

async function triggerCheck() {
  if (checking.value) return;
  checking.value = true;
  try {
    status.value = await api.checkNow();
    toast("IP address checked", "ok");
  } catch (e) {
    toast(`Check failed: ${e}`, "error");
  } finally {
    checking.value = false;
  }
}

async function toggleFullscreen() {
  try {
    const fs = await appWindow.isFullscreen();
    await appWindow.setFullscreen(!fs);
    isFullscreen.value = !fs;
  } catch {
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen().catch(() => {});
      isFullscreen.value = true;
    } else {
      await document.exitFullscreen().catch(() => {});
      isFullscreen.value = false;
    }
  }
}

async function copyIp(ip?: string | null) {
  const target = ip || status.value?.public_ip;
  if (!target) return;
  await navigator.clipboard.writeText(target);
  toast(`Copied ${target}`, "ok");
}

onMounted(async () => {
  const s = await api.settings();
  setTheme(s.theme);
  status.value = await api.current();

  try {
    isFullscreen.value = await appWindow.isFullscreen();
  } catch {}

  unlisten.push(await api.onChecked((s) => {
    status.value = s;
  }));

  unlisten.push(await api.onEvent((e) => {
    eventSeq.value++;
    if (e.kind === "change") {
      toast(`${kindLabel(e.kind)}: ${e.prev_public_ip ?? "?"} → ${e.public_ip}`, "info", 6000);
    }
  }));

  unlisten.push(await api.onCopyIp(async (ip) => {
    await copyIp(ip);
  }));

  window.addEventListener("keydown", onKey);

  // Auto-collapse sidebar on narrow screens
  if (window.innerWidth < 768) {
    sidebarCollapsed.value = true;
  }
});

onUnmounted(() => {
  unlisten.forEach((u) => u());
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <div class="flex flex-col h-full w-full bg-bg text-fg select-none overflow-hidden font-sans">
    <!-- Desktop Native Top Window Bar / Header -->
    <header
      data-tauri-drag-region
      class="h-12 border-b border-line bg-surface/75 backdrop-blur-md shrink-0 flex items-center justify-between px-3.5 z-30 titlebar-drag"
    >
      <!-- Left: Logo & Sidebar Toggle & Status Pill -->
      <div class="flex items-center gap-3 no-drag">
        <button
          class="btn-icon text-fg2 hover:text-fg hover:bg-surface2/80 rounded-md p-1.5 transition-colors"
          title="Toggle sidebar (⌘B)"
          @click="sidebarCollapsed = !sidebarCollapsed"
        >
          <Icon name="menu" :size="17" />
        </button>

        <div class="flex items-center gap-2">
          <div class="h-6 w-6 rounded-md bg-accent/15 text-accent flex items-center justify-center font-bold text-[12px] tracking-tight border border-accent/20">
            IP
          </div>
          <span class="font-semibold text-[13px] tracking-tight hidden sm:inline">IP Tracker</span>
        </div>

        <div class="h-4 w-px bg-line/80 mx-1 hidden sm:block"></div>

        <!-- Connection status badge -->
        <div
          class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[11px] font-medium transition-colors"
          :class="
            status?.checking || checking
              ? 'bg-accent/10 text-accent border border-accent/20'
              : status?.paused
              ? 'bg-surface2 text-fg3'
              : status?.online
              ? 'bg-ok/10 text-ok border border-ok/20'
              : 'bg-danger/10 text-danger border border-danger/20'
          "
        >
          <span class="relative flex h-2 w-2">
            <span
              v-if="status?.checking || checking"
              class="animate-ping absolute inline-flex h-full w-full rounded-full bg-accent opacity-75"
            ></span>
            <span
              class="relative inline-flex rounded-full h-2 w-2"
              :class="
                status?.checking || checking
                  ? 'bg-accent'
                  : status?.paused
                  ? 'bg-fg3'
                  : status?.online
                  ? 'bg-ok'
                  : 'bg-danger'
              "
            ></span>
          </span>
          <span class="capitalize">{{
            status?.checking || checking
              ? 'Checking…'
              : status?.paused
              ? 'Paused'
              : status?.online
              ? 'Online'
              : 'Offline'
          }}</span>
        </div>
      </div>

      <!-- Right: Action Buttons & Fullscreen Toggle -->
      <div class="flex items-center gap-1.5 no-drag">
        <button
          class="btn btn-sm btn-secondary h-8 px-2.5"
          :disabled="status?.checking || checking"
          title="Check public address now (⌘R)"
          @click="triggerCheck"
        >
          <Icon
            name="refresh"
            :size="14"
            :class="{ 'animate-spin text-accent': status?.checking || checking }"
          />
          <span class="hidden md:inline">{{ status?.checking || checking ? 'Checking…' : 'Check now' }}</span>
        </button>

        <button
          class="btn-icon p-1.5 text-fg2 hover:text-fg hover:bg-surface2 rounded-md transition-colors"
          :title="isFullscreen ? 'Exit Fullscreen (F11)' : 'Enter Fullscreen (F11)'"
          @click="toggleFullscreen"
        >
          <Icon :name="isFullscreen ? 'fullscreen-exit' : 'fullscreen'" :size="16" />
        </button>
      </div>
    </header>

    <!-- Main Workspace Area -->
    <div class="flex flex-1 min-h-0 overflow-hidden relative">
      <!-- Responsive Desktop Sidebar -->
      <aside
        class="border-r border-line bg-surface flex flex-col shrink-0 transition-all duration-200 ease-in-out z-20"
        :class="sidebarCollapsed ? 'w-16' : 'w-56 lg:w-60'"
      >
        <!-- Nav items -->
        <nav class="p-2 space-y-1 flex-1 overflow-y-auto">
          <button
            v-for="t in tabs"
            :key="t.id"
            class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-150 text-[13px] group relative"
            :class="
              tab === t.id
                ? 'bg-accent/10 text-accent font-semibold shadow-xs'
                : 'text-fg2 hover:bg-surface2/70 hover:text-fg font-medium'
            "
            :title="sidebarCollapsed ? t.label : undefined"
            @click="tab = t.id"
          >
            <Icon
              :name="t.icon"
              :size="18"
              class="shrink-0 transition-colors"
              :class="tab === t.id ? 'text-accent' : 'text-fg3 group-hover:text-fg2'"
            />
            <span v-if="!sidebarCollapsed" class="truncate">{{ t.label }}</span>
            <kbd
              v-if="!sidebarCollapsed"
              class="ml-auto text-[10px] text-fg3/80 font-mono px-1.5 py-0.5 rounded border border-line/60 bg-surface2/60"
            >
              ⌘{{ t.key }}
            </kbd>

            <!-- Active indicator border pill -->
            <div
              v-if="tab === t.id"
              class="absolute left-0 top-1.5 bottom-1.5 w-1 rounded-r-full bg-accent"
            ></div>
          </button>
        </nav>

        <!-- Current IP glance pill at sidebar bottom -->
        <div class="p-3 border-t border-line/80 bg-surface2/30">
          <div
            v-if="!sidebarCollapsed"
            class="card p-2.5 bg-surface border-line/70 flex flex-col gap-1 cursor-pointer hover:border-accent/50 transition-colors group"
            title="Click to copy public IP"
            @click="copyIp()"
          >
            <div class="flex items-center justify-between text-[11px] text-fg3 font-medium">
              <span>Public IP</span>
              <Icon name="copy" :size="12" class="opacity-0 group-hover:opacity-100 text-accent transition-opacity" />
            </div>
            <p class="num font-semibold text-[13px] text-fg tracking-tight truncate">
              {{ status?.public_ip ?? (status?.paused ? 'Paused' : 'Offline') }}
            </p>
            <p v-if="status?.isp" class="text-[11px] text-fg2 truncate">
              {{ status.isp }}
            </p>
          </div>
          <button
            v-else
            class="w-full flex items-center justify-center p-2 rounded-lg text-fg2 hover:text-accent hover:bg-surface2 transition-colors"
            title="Copy Public IP"
            @click="copyIp()"
          >
            <Icon name="copy" :size="18" />
          </button>
        </div>
      </aside>

      <!-- Scrollable Main Content (Scales fluidly to Fullscreen) -->
      <main class="flex-1 min-w-0 overflow-y-auto overflow-x-hidden flex flex-col bg-bg">
        <div class="flex-1 w-full max-w-7xl 2xl:max-w-[1600px] mx-auto p-4 sm:p-6 lg:p-8 2xl:p-10 transition-all duration-150">
          <Dashboard v-if="tab === 'dashboard'" :status="status" :event-seq="eventSeq" />
          <History v-else-if="tab === 'history'" :event-seq="eventSeq" />
          <Reports v-else-if="tab === 'reports'" :event-seq="eventSeq" />
          <SettingsView v-else />
        </div>
      </main>
    </div>

    <!-- Modals & Notifications -->
    <Toasts />
    <ConfirmDialog />
  </div>
</template>
