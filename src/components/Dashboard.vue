<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import PageHeader from "./ui/PageHeader.vue";
import LabelEditor from "./ui/LabelEditor.vue";
import { api, type CurrentStatus, type IpEvent } from "../lib/api";
import { fmtDateTime, fmtTime, sinceNow, untilNow, kindLabel, kindClass } from "../lib/format";
import { toast } from "../lib/toast";

const props = defineProps<{ status: CurrentStatus | null; eventSeq: number }>();

const recent = ref<IpEvent[]>([]);
const loading = ref(true);
const tick = ref(0);
let timer: number | undefined;

async function load() {
  recent.value = await api.history({ limit: 8 });
  loading.value = false;
}

async function checkNow() {
  try {
    await api.checkNow();
  } catch (e) {
    toast(`Check failed: ${e}`, "error");
  }
}

async function togglePause() {
  const paused = await api.setPaused(!props.status?.paused);
  toast(paused ? "Tracking paused" : "Tracking resumed", "info");
}

async function copy(text: string | null) {
  if (!text) return;
  await navigator.clipboard.writeText(text);
  toast(`Copied ${text}`, "ok");
}

watch(() => props.eventSeq, load);
onMounted(() => {
  load();
  timer = window.setInterval(() => tick.value++, 15_000);
});
onUnmounted(() => clearInterval(timer));
</script>

<template>
  <PageHeader title="Overview" :subtitle="status?.paused ? 'Tracking is paused.' : 'Watching your public address in the background.'">
    <button class="btn btn-secondary" @click="togglePause">{{ status?.paused ? 'Resume' : 'Pause' }}</button>
    <button class="btn btn-primary" :disabled="status?.checking" @click="checkNow">
      {{ status?.checking ? 'Checking…' : 'Check now' }}
    </button>
  </PageHeader>

  <section class="card p-6 mb-5">
    <div class="flex flex-wrap items-start justify-between gap-6">
      <div class="min-w-0">
        <div class="flex items-center gap-3 mb-1.5">
          <span class="text-fg2">IPv4</span>
          <LabelEditor v-if="status?.public_ip" :ip="status.public_ip" :label="status.label" />
        </div>
        <button
          class="num text-4xl md:text-5xl font-semibold tracking-tight text-left hover:text-accent transition-colors truncate max-w-full"
          :class="{ 'text-fg3': !status?.public_ip }"
          title="Click to copy"
          :disabled="!status?.public_ip"
          @click="copy(status?.public_ip ?? null)"
        >
          {{ status?.public_ip ?? (status?.online === false && status?.last_checked ? 'Offline' : '—') }}
        </button>
        <p class="mt-3 text-fg2">
          <template v-if="status?.isp">{{ status.isp }}</template>
          <template v-if="status?.city || status?.country">
            <span v-if="status?.isp"> · </span>{{ [status?.city, status?.country].filter(Boolean).join(", ") }}
          </template>
          <template v-if="!status?.isp && !status?.country && status?.public_ip">Provider lookup is off or unavailable.</template>
        </p>
      </div>

      <dl class="grid grid-cols-2 gap-x-8 gap-y-3 text-[13px] shrink-0" :key="tick">
        <div>
          <dt class="text-fg3">On this address</dt>
          <dd class="num">{{ status?.since ? sinceNow(status.since) : '—' }}</dd>
        </div>
        <div>
          <dt class="text-fg3">Last checked</dt>
          <dd class="num">{{ status?.last_checked ? fmtTime(status.last_checked) : 'never' }}</dd>
        </div>
        <div>
          <dt class="text-fg3">Next check</dt>
          <dd class="num">{{ status?.paused ? 'paused' : untilNow(status?.next_check) }}</dd>
        </div>
        <div>
          <dt class="text-fg3">Status</dt>
          <dd>
            <span class="chip" :class="status?.paused ? 'bg-surface2 text-fg2' : status?.online ? 'bg-ok/15 text-ok' : 'bg-danger/15 text-danger'">
              {{ status?.paused ? 'Paused' : status?.online ? 'Online' : 'Offline' }}
            </span>
          </dd>
        </div>
      </dl>
    </div>

    <div v-if="status?.public_ipv6" class="mt-6 pt-5 border-t border-line flex items-center gap-3">
      <span class="text-fg2 w-10">IPv6</span>
      <button class="num selectable hover:text-accent transition-colors truncate" title="Click to copy" @click="copy(status.public_ipv6)">
        {{ status.public_ipv6 }}
      </button>
    </div>
  </section>

  <div class="grid md:grid-cols-[1fr_2fr] gap-5">
    <section class="card p-5">
      <h2 class="font-medium mb-3">Local addresses</h2>
      <ul v-if="status?.local_ips?.length" class="space-y-1.5">
        <li v-for="ip in status.local_ips" :key="ip">
          <button class="num selectable text-fg2 hover:text-accent transition-colors truncate max-w-full" title="Click to copy" @click="copy(ip)">{{ ip }}</button>
        </li>
      </ul>
      <p v-else class="text-fg3">No active interfaces.</p>
    </section>

    <section class="card p-5">
      <h2 class="font-medium mb-3">Recent events</h2>
      <div v-if="loading" class="space-y-2">
        <div v-for="i in 4" :key="i" class="skeleton h-5"></div>
      </div>
      <p v-else-if="recent.length === 0" class="text-fg3">Nothing logged yet. The first check writes a starting record.</p>
      <ul v-else class="divide-y divide-line">
        <li v-for="e in recent" :key="e.id" class="py-2 flex items-center gap-3">
          <span class="text-fg3 num text-[13px] w-36 shrink-0">{{ fmtDateTime(e.ts) }}</span>
          <span class="chip shrink-0" :class="kindClass(e.kind)">{{ kindLabel(e.kind) }}</span>
          <span class="num truncate selectable">
            <template v-if="e.kind === 'change' && e.prev_public_ip">
              <span class="text-fg3">{{ e.prev_public_ip }}</span> → {{ e.public_ip }}
            </template>
            <template v-else>{{ e.public_ip ?? e.prev_public_ip ?? '—' }}</template>
          </span>
        </li>
      </ul>
    </section>
  </div>
</template>
