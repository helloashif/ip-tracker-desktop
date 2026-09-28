<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { enable, disable } from "@tauri-apps/plugin-autostart";
import { openPath } from "@tauri-apps/plugin-opener";
import PageHeader from "./ui/PageHeader.vue";
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
  watch(s, () => { dirty.value = true; }, { deep: true });
  watch(() => s.value?.theme, (t) => t && setTheme(t));
});

async function save() {
  if (!s.value) return;
  saving.value = true;
  try {
    if (s.value.autostart) await enable(); else await disable();
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
  { v: 1, l: "1 minute" }, { v: 2, l: "2 minutes" }, { v: 5, l: "5 minutes" }, { v: 10, l: "10 minutes" },
  { v: 15, l: "15 minutes" }, { v: 30, l: "30 minutes" }, { v: 60, l: "1 hour" },
];
const retentions = [
  { v: 0, l: "Forever" }, { v: 30, l: "30 days" }, { v: 90, l: "90 days" }, { v: 365, l: "1 year" },
];
</script>

<template>
  <PageHeader title="Settings">
    <span v-if="dirty" class="text-fg3 text-[13px]">Unsaved changes</span>
    <button class="btn btn-primary" :disabled="!dirty || saving" @click="save">{{ saving ? 'Saving…' : 'Save changes' }}</button>
  </PageHeader>

  <div v-if="s" class="max-w-2xl space-y-5">
    <section class="card p-5">
      <h2 class="font-medium mb-4">Tracking</h2>
      <div class="space-y-4">
        <label class="block">
          <span class="field-label">Check every</span>
          <select v-model.number="s.interval_minutes" class="input w-48">
            <option v-for="i in intervals" :key="i.v" :value="i.v">{{ i.l }}</option>
          </select>
          <p class="field-hint">Shorter intervals catch changes sooner but make more requests to the lookup services.</p>
        </label>
        <label class="flex items-start gap-3">
          <input v-model="s.track_ipv6" type="checkbox" class="mt-1 accent-accent" />
          <span>
            <span class="block">Also track IPv6</span>
            <span class="block text-fg2 text-[13px]">Logs a change when either address changes. Turn off if you have no IPv6 connectivity.</span>
          </span>
        </label>
        <label class="flex items-start gap-3">
          <input v-model="s.autostart" type="checkbox" class="mt-1 accent-accent" />
          <span>
            <span class="block">Start when I log in</span>
            <span class="block text-fg2 text-[13px]">Runs in the tray so changes are logged while you work.</span>
          </span>
        </label>
        <label class="flex items-start gap-3">
          <input v-model="s.start_minimized" type="checkbox" class="mt-1 accent-accent" />
          <span class="block">Start minimized to the tray</span>
        </label>
      </div>
    </section>

    <section class="card p-5">
      <h2 class="font-medium mb-4">Notifications and privacy</h2>
      <div class="space-y-4">
        <label class="flex items-start gap-3">
          <input v-model="s.notifications" type="checkbox" class="mt-1 accent-accent" />
          <span>
            <span class="block">Notify me on changes</span>
            <span class="block text-fg2 text-[13px]">System notification when the address changes or the connection drops.</span>
          </span>
        </label>
        <label class="flex items-start gap-3">
          <input v-model="s.geo_lookup" type="checkbox" class="mt-1 accent-accent" />
          <span>
            <span class="block">Look up provider and location</span>
            <span class="block text-fg2 text-[13px]">Sends each new public address to ip-api.com to get the ISP and city. Nothing else is sent.</span>
          </span>
        </label>
      </div>
    </section>

    <section class="card p-5">
      <h2 class="font-medium mb-1">Integrations</h2>
      <p class="text-fg2 text-[13px] mb-4">Run something when the address changes — update dynamic DNS, ping a chat channel, or write to a log.</p>
      <div class="space-y-5">
        <div>
          <label class="field-label" for="webhook">Webhook URL</label>
          <div class="flex gap-2">
            <input id="webhook" v-model.trim="s.webhook_url" type="url" placeholder="https://example.com/hooks/ip" class="input" />
            <button class="btn btn-secondary" :disabled="!s.webhook_url || testing" @click="testWebhook">{{ testing ? 'Sending…' : 'Send test' }}</button>
          </div>
          <p class="field-hint">POSTs a JSON body with <code class="num">event</code>, <code class="num">public_ipv4</code>, <code class="num">previous_ipv4</code>, <code class="num">isp</code> and more.</p>
        </div>
        <div>
          <label class="field-label" for="hook">Shell command</label>
          <input id="hook" v-model="s.hook_command" type="text" placeholder="e.g. /home/me/update-dns.sh" class="input num" spellcheck="false" />
          <p class="field-hint">Runs after each event with <code class="num">IP_EVENT</code>, <code class="num">IP_NEW</code>, <code class="num">IP_OLD</code>, <code class="num">IP_NEW_V6</code>, <code class="num">IP_ISP</code> and <code class="num">IP_LABEL</code> as environment variables.</p>
        </div>
      </div>
    </section>

    <section class="card p-5">
      <h2 class="font-medium mb-4">Appearance</h2>
      <div class="inline-flex rounded-md border border-line overflow-hidden">
        <button
          v-for="t in (['system', 'light', 'dark'] as const)" :key="t"
          class="px-4 py-1.5 capitalize transition-colors"
          :class="s.theme === t ? 'bg-surface2 text-fg font-medium' : 'text-fg2 hover:bg-surface2/60'"
          @click="s.theme = t"
        >{{ t }}</button>
      </div>
    </section>

    <section class="card p-5">
      <h2 class="font-medium mb-4">Data</h2>
      <div class="space-y-4">
        <label class="block">
          <span class="field-label">Keep history for</span>
          <select v-model.number="s.retention_days" class="input w-48">
            <option v-for="r in retentions" :key="r.v" :value="r.v">{{ r.l }}</option>
          </select>
          <p class="field-hint">Older events are removed automatically after each check.</p>
        </label>
        <dl v-if="info" class="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1.5 text-[13px]">
          <dt class="text-fg3">Events stored</dt><dd class="num">{{ info.event_count }}</dd>
          <dt class="text-fg3">Database size</dt><dd class="num">{{ fmtBytes(info.db_size_bytes) }}</dd>
          <dt class="text-fg3">Location</dt>
          <dd class="num selectable break-all">
            <button class="hover:text-accent text-left" title="Open folder" @click="openPath(info.data_dir)">{{ info.data_dir }}</button>
          </dd>
        </dl>
        <button class="btn btn-danger btn-sm" @click="clearAll">Delete all history</button>
      </div>
    </section>

    <section class="card p-5">
      <h2 class="font-medium mb-2">About</h2>
      <p class="text-fg2 text-[13px]">
        IP Tracker {{ info?.version }} · 
      </p>
    </section>
  </div>
</template>
