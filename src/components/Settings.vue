<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { enable, disable } from "@tauri-apps/plugin-autostart";
import { openPath } from "@tauri-apps/plugin-opener";
import PageHeader from "./ui/PageHeader.vue";
import Icon from "./ui/Icon.vue";
import Select from "./ui/Select.vue";
import { api, type Settings, type AppInfo } from "../lib/api";
import { setTheme } from "../lib/theme";
import { toast } from "../lib/toast";
import { confirm } from "../lib/confirm";
import { fmtBytes } from "../lib/format";

const s = ref<Settings | null>(null);
const info = ref<AppInfo | null>(null);
const dirty = ref(false);
const saving = ref(false);
const testing = ref(false);

onMounted(async () => {
  s.value = await api.settings();
  info.value = await api.appInfo();
  watch(
    s,
    () => {
      dirty.value = true;
    },
    { deep: true }
  );
  watch(
    () => s.value?.theme,
    (t) => t && setTheme(t)
  );
});

async function save() {
  if (!s.value) return;
  saving.value = true;
  try {
    if (s.value.autostart) await enable();
    else await disable();
    s.value = await api.saveSettings(s.value);
    dirty.value = false;
    toast("Settings saved", "ok");
  } catch (e) {
    toast(`Couldn't save: ${e}`, "error");
  } finally {
    saving.value = false;
  }
}

async function testWebhook() {
  if (!s.value?.webhook_url) return;
  testing.value = true;
  try {
    const r = await api.testWebhook(s.value.webhook_url);
    toast(`Webhook responded: ${r}`, "ok");
  } catch (e) {
    toast(`Webhook failed: ${e}`, "error");
  } finally {
    testing.value = false;
  }
}

async function clearAll() {
  const ok = await confirm({
    title: "Delete all history?",
    body: "Every logged event will be removed. Labels and settings are kept.",
    confirmLabel: "Delete everything",
    danger: true,
  });
  if (!ok) return;
  await api.clearHistory();
  info.value = await api.appInfo();
  toast("History cleared", "ok");
}

const intervals = [
  { v: 1, l: "1 minute" },
  { v: 2, l: "2 minutes" },
  { v: 5, l: "5 minutes" },
  { v: 10, l: "10 minutes" },
  { v: 15, l: "15 minutes" },
  { v: 30, l: "30 minutes" },
  { v: 60, l: "1 hour" },
];

const retentions = [
  { v: 0, l: "Forever" },
  { v: 30, l: "30 days" },
  { v: 90, l: "90 days" },
  { v: 365, l: "1 year" },
];
</script>

<template>
  <div class="space-y-6">
    <PageHeader title="Settings" subtitle="Customize tracker polling intervals, notifications, and storage.">
      <span v-if="dirty" class="text-accent text-[12px] font-semibold animate-pulse mr-2">
        • Unsaved changes
      </span>
      <button class="btn btn-primary h-9" :disabled="!dirty || saving" @click="save">
        <Icon v-if="saving" name="refresh" :size="14" class="animate-spin" />
        <span>{{ saving ? 'Saving…' : 'Save changes' }}</span>
      </button>
    </PageHeader>

    <div v-if="s" class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
      <!-- Section 1: Tracking Configuration -->
      <section class="card p-5 space-y-4">
        <div class="flex items-center gap-2 pb-2 border-b border-line/60">
          <Icon name="clock" :size="16" class="text-accent" />
          <h2 class="font-semibold text-[14px]">Tracking Schedule</h2>
        </div>

        <div>
          <label class="field-label" for="interval-select">Polling Frequency</label>
          <Select
            id="interval-select"
            v-model="s.interval_minutes"
            :options="intervals.map((i) => ({ value: i.v, label: i.l }))"
          />
          <p class="field-hint">
            Frequency of IP detection checks. Shorter intervals catch network handovers faster.
          </p>
        </div>

        <!-- Desktop Toggle: Also Track IPv6 -->
        <div class="flex items-start justify-between gap-4 pt-3 border-t border-line/60">
          <div class="space-y-0.5">
            <span class="text-fg font-medium text-[13px] block">Track Public IPv6</span>
            <span class="text-fg2 text-[12px] block">Detect and log changes when your IPv6 address changes.</span>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="s.track_ipv6"
            class="toggle-switch shrink-0"
            :class="s.track_ipv6 ? 'bg-accent' : 'bg-surface2 border-line'"
            @click="s.track_ipv6 = !s.track_ipv6"
          >
            <span
              class="toggle-switch-thumb"
              :class="s.track_ipv6 ? 'translate-x-5' : 'translate-x-0'"
            ></span>
          </button>
        </div>

        <!-- Desktop Toggle: Autostart -->
        <div class="flex items-start justify-between gap-4 pt-3 border-t border-line/60">
          <div class="space-y-0.5">
            <span class="text-fg font-medium text-[13px] block">Start at Login</span>
            <span class="text-fg2 text-[12px] block">Launch automatically on system boot in the background tray.</span>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="s.autostart"
            class="toggle-switch shrink-0"
            :class="s.autostart ? 'bg-accent' : 'bg-surface2 border-line'"
            @click="s.autostart = !s.autostart"
          >
            <span
              class="toggle-switch-thumb"
              :class="s.autostart ? 'translate-x-5' : 'translate-x-0'"
            ></span>
          </button>
        </div>

        <!-- Desktop Toggle: Start Minimized -->
        <div class="flex items-start justify-between gap-4 pt-3 border-t border-line/60">
          <div class="space-y-0.5">
            <span class="text-fg font-medium text-[13px] block">Start Minimized to Tray</span>
            <span class="text-fg2 text-[12px] block">Keep the main window hidden when the app boots up.</span>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="s.start_minimized"
            class="toggle-switch shrink-0"
            :class="s.start_minimized ? 'bg-accent' : 'bg-surface2 border-line'"
            @click="s.start_minimized = !s.start_minimized"
          >
            <span
              class="toggle-switch-thumb"
              :class="s.start_minimized ? 'translate-x-5' : 'translate-x-0'"
            ></span>
          </button>
        </div>
      </section>

      <!-- Section 2: Notifications & Geolocation -->
      <section class="card p-5 space-y-4">
        <div class="flex items-center gap-2 pb-2 border-b border-line/60">
          <Icon name="shield" :size="16" class="text-accent" />
          <h2 class="font-semibold text-[14px]">Notifications & Geolocation</h2>
        </div>

        <!-- Desktop Toggle: Notifications -->
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-0.5">
            <span class="text-fg font-medium text-[13px] block">Desktop System Notifications</span>
            <span class="text-fg2 text-[12px] block">Receive desktop banner alerts on IP change or disconnection.</span>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="s.notifications"
            class="toggle-switch shrink-0"
            :class="s.notifications ? 'bg-accent' : 'bg-surface2 border-line'"
            @click="s.notifications = !s.notifications"
          >
            <span
              class="toggle-switch-thumb"
              :class="s.notifications ? 'translate-x-5' : 'translate-x-0'"
            ></span>
          </button>
        </div>

        <!-- Desktop Toggle: Geo lookup -->
        <div class="flex items-start justify-between gap-4 pt-3 border-t border-line/60">
          <div class="space-y-0.5">
            <span class="text-fg font-medium text-[13px] block">Lookup Provider and Location</span>
            <span class="text-fg2 text-[12px] block">
              Resolves ISP organization and approximate city for your address.
            </span>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="s.geo_lookup"
            class="toggle-switch shrink-0"
            :class="s.geo_lookup ? 'bg-accent' : 'bg-surface2 border-line'"
            @click="s.geo_lookup = !s.geo_lookup"
          >
            <span
              class="toggle-switch-thumb"
              :class="s.geo_lookup ? 'translate-x-5' : 'translate-x-0'"
            ></span>
          </button>
        </div>

        <!-- Appearance Section -->
        <div class="pt-3 border-t border-line/60">
          <span class="field-label">Theme Appearance</span>
          <div class="inline-flex rounded-lg border border-line bg-surface2/40 p-1 gap-1 w-full">
            <button
              v-for="t in (['system', 'light', 'dark'] as const)"
              :key="t"
              class="flex-1 py-1.5 capitalize rounded-md text-[12px] font-medium transition-all text-center"
              :class="s.theme === t ? 'bg-surface text-fg font-semibold shadow-xs' : 'text-fg2 hover:text-fg'"
              @click="s.theme = t"
            >
              {{ t }}
            </button>
          </div>
        </div>
      </section>

      <!-- Section 3: Webhooks & Shell Integration -->
      <section class="card p-5 space-y-4 lg:col-span-2">
        <div class="flex items-center gap-2 pb-2 border-b border-line/60">
          <Icon name="network" :size="16" class="text-accent" />
          <div>
            <h2 class="font-semibold text-[14px]">Automation & Webhooks</h2>
            <p class="text-fg3 text-[12px]">Trigger webhooks or local shell scripts whenever your IP address rotates.</p>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
          <div>
            <label class="field-label" for="webhook">Webhook Endpoint (HTTP POST)</label>
            <div class="flex gap-2">
              <input
                id="webhook"
                v-model.trim="s.webhook_url"
                type="url"
                placeholder="https://example.com/api/ip-webhook"
                class="input"
              />
              <button
                class="btn btn-secondary h-9 px-3 shrink-0"
                :disabled="!s.webhook_url || testing"
                @click="testWebhook"
              >
                {{ testing ? 'Sending…' : 'Test' }}
              </button>
            </div>
            <p class="field-hint">
              Sends JSON payload containing <code class="num">public_ipv4</code>, <code class="num">previous_ipv4</code>, and <code class="num">isp</code>.
            </p>
          </div>

          <div>
            <label class="field-label" for="hook">Shell Script Hook</label>
            <input
              id="hook"
              v-model="s.hook_command"
              type="text"
              placeholder="/home/user/scripts/update-ddns.sh"
              class="input num text-[12px]"
              spellcheck="false"
            />
            <p class="field-hint">
              Executes with environment variables: <code class="num">$IP_NEW</code>, <code class="num">$IP_OLD</code>, <code class="num">$IP_ISP</code>.
            </p>
          </div>
        </div>
      </section>

      <!-- Section 4: Data & Local Storage -->
      <section class="card p-5 space-y-4 lg:col-span-2">
        <div class="flex items-center justify-between pb-2 border-b border-line/60">
          <div class="flex items-center gap-2">
            <Icon name="trash" :size="16" class="text-accent" />
            <h2 class="font-semibold text-[14px]">Data Retention & Storage</h2>
          </div>
          <button class="btn btn-danger btn-sm h-8" @click="clearAll">
            Delete all history
          </button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-5 items-center">
          <div>
            <label class="field-label">Retention Period</label>
            <Select v-model="s.retention_days" :options="retentions.map((r) => ({ value: r.v, label: r.l }))" />
            <p class="field-hint">Old events beyond retention window are pruned automatically.</p>
          </div>

          <div v-if="info" class="bg-surface2/40 p-4 rounded-xl border border-line/50 text-[12px] space-y-1.5">
            <div class="flex justify-between">
              <span class="text-fg3">SQLite Database Size</span>
              <span class="num font-semibold">{{ fmtBytes(info.db_size_bytes) }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-fg3">Stored Event Records</span>
              <span class="num font-semibold">{{ info.event_count }} events</span>
            </div>
            <div class="flex justify-between items-center pt-1 border-t border-line/40">
              <span class="text-fg3">Storage Path</span>
              <button
                class="num text-accent hover:underline truncate max-w-xs text-right cursor-pointer"
                title="Open directory"
                @click="openPath(info.data_dir)"
              >
                {{ info.data_dir }}
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
