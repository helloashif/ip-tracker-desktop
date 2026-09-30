<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import PageHeader from "./ui/PageHeader.vue";
import LabelEditor from "./ui/LabelEditor.vue";
import Icon from "./ui/Icon.vue";
import { api, type CurrentStatus, type IpEvent } from "../lib/api";
import { fmtDateTime, fmtTime, sinceNow, untilNow, kindLabel, kindClass } from "../lib/format";
import { toast } from "../lib/toast";

const props = defineProps<{ status: CurrentStatus | null; eventSeq: number }>();

const recent = ref<IpEvent[]>([]);
const loading = ref(true);
const tick = ref(0);
const copiedField = ref<string | null>(null);
let timer: number | undefined;

async function load() {
  recent.value = await api.history({ limit: 10 });
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

async function copy(text: string | null, fieldKey?: string) {
  if (!text) return;
  await navigator.clipboard.writeText(text);
  if (fieldKey) {
    copiedField.value = fieldKey;
    setTimeout(() => {
      if (copiedField.value === fieldKey) copiedField.value = null;
    }, 2000);
  }
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
  <div class="space-y-6">
    <PageHeader
      title="Overview"
      :subtitle="status?.paused ? 'Tracking is paused.' : 'Monitoring your public address continuously in the background.'"
    >
      <button class="btn btn-secondary h-9" @click="togglePause">
        <Icon :name="status?.paused ? 'play' : 'pause'" :size="14" />
        <span>{{ status?.paused ? 'Resume tracking' : 'Pause tracking' }}</span>
      </button>
      <button class="btn btn-primary h-9" :disabled="status?.checking" @click="checkNow">
        <Icon name="refresh" :size="14" :class="{ 'animate-spin': status?.checking }" />
        <span>{{ status?.checking ? 'Checking…' : 'Check now' }}</span>
      </button>
    </PageHeader>

    <!-- Primary Hero IP Card -->
    <section class="card p-6 md:p-8 bg-surface/90 relative overflow-hidden border-line/90 shadow-sm">
      <!-- Decorative ambient background glow -->
      <div
        class="absolute -top-24 -right-24 w-80 h-80 rounded-full blur-3xl opacity-20 pointer-events-none"
        :class="status?.online ? 'bg-accent' : 'bg-danger'"
      ></div>

      <div class="flex flex-col lg:flex-row lg:items-start justify-between gap-6 relative z-10">
        <!-- Main IP details -->
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2.5 mb-2">
            <span class="text-[12px] font-semibold tracking-wider uppercase text-fg3 bg-surface2/80 px-2 py-0.5 rounded">
              IPv4 Public Address
            </span>
            <LabelEditor v-if="status?.public_ip" :ip="status.public_ip" :label="status.label" />
          </div>

          <!-- Main IP Click to Copy -->
          <div class="flex items-center gap-3 my-2 group">
            <button
              class="num text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight text-left hover:text-accent transition-colors truncate max-w-full cursor-pointer focus:outline-none focus:text-accent"
              :class="{ 'text-fg3': !status?.public_ip }"
              title="Click to copy IP"
              :disabled="!status?.public_ip"
              @click="copy(status?.public_ip ?? null, 'v4')"
            >
              {{ status?.public_ip ?? (status?.online === false && status?.last_checked ? 'Offline' : '—') }}
            </button>
            <button
              v-if="status?.public_ip"
              class="btn-icon text-fg3 group-hover:text-accent hover:bg-surface2 transition-all"
              title="Copy to clipboard"
              @click="copy(status.public_ip, 'v4')"
            >
              <Icon :name="copiedField === 'v4' ? 'check' : 'copy'" :size="18" :class="{ 'text-ok': copiedField === 'v4' }" />
            </button>
          </div>

          <!-- Provider & Location badges -->
          <div class="flex flex-wrap items-center gap-2 mt-4 text-[13px]">
            <div
              v-if="status?.isp"
              class="inline-flex items-center gap-1.5 px-3 py-1 rounded-md bg-surface2/70 text-fg border border-line/60 font-medium"
            >
              <Icon name="globe" :size="14" class="text-accent" />
              <span>{{ status.isp }}</span>
            </div>

            <div
              v-if="status?.city || status?.country"
              class="inline-flex items-center gap-1.5 px-3 py-1 rounded-md bg-surface2/70 text-fg2 border border-line/60"
            >
              <Icon name="map-pin" :size="14" class="text-fg3" />
              <span>{{ [status?.city, status?.country].filter(Boolean).join(", ") }}</span>
            </div>

            <span v-if="!status?.isp && !status?.country && status?.public_ip" class="text-fg3 text-[12px]">
              Provider lookup is off or unavailable.
            </span>
          </div>
        </div>

        <!-- Telemetry Metadata Tiles -->
        <dl
          class="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-2 gap-3 lg:gap-4 shrink-0 text-[13px] bg-surface2/40 p-4 rounded-xl border border-line/50 min-w-[280px]"
          :key="tick"
        >
          <div>
            <dt class="text-fg3 text-[11px] font-medium uppercase tracking-wider mb-0.5">Active Since</dt>
            <dd class="num font-semibold text-fg">{{ status?.since ? sinceNow(status.since) : '—' }}</dd>
          </div>

          <div>
            <dt class="text-fg3 text-[11px] font-medium uppercase tracking-wider mb-0.5">Last Checked</dt>
            <dd class="num text-fg">{{ status?.last_checked ? fmtTime(status.last_checked) : 'never' }}</dd>
          </div>

          <div>
            <dt class="text-fg3 text-[11px] font-medium uppercase tracking-wider mb-0.5">Next Check</dt>
            <dd class="num text-fg">{{ status?.paused ? 'paused' : untilNow(status?.next_check) }}</dd>
          </div>

          <div>
            <dt class="text-fg3 text-[11px] font-medium uppercase tracking-wider mb-0.5">Network State</dt>
            <dd class="mt-0.5">
              <span
                class="chip"
                :class="
                  status?.paused
                    ? 'bg-surface2 text-fg3'
                    : status?.online
                    ? 'bg-ok/15 text-ok border-ok/30'
                    : 'bg-danger/15 text-danger border-danger/30'
                "
              >
                {{ status?.paused ? 'Paused' : status?.online ? 'Online' : 'Offline' }}
              </span>
            </dd>
          </div>
        </dl>
      </div>

      <!-- IPv6 Bar (if present) -->
      <div
        v-if="status?.public_ipv6"
        class="mt-6 pt-4 border-t border-line/80 flex flex-wrap items-center justify-between gap-3 text-[13px]"
      >
        <div class="flex items-center gap-2.5">
          <span class="text-[11px] font-semibold tracking-wider uppercase text-fg3 bg-surface2/60 px-1.5 py-0.5 rounded">
            IPv6
          </span>
          <button
            class="num text-fg hover:text-accent font-medium truncate max-w-xl transition-colors cursor-pointer text-left"
            title="Click to copy IPv6"
            @click="copy(status.public_ipv6, 'v6')"
          >
            {{ status.public_ipv6 }}
          </button>
        </div>
        <button
          class="btn btn-sm btn-ghost h-7 px-2 text-fg2"
          @click="copy(status.public_ipv6, 'v6')"
        >
          <Icon :name="copiedField === 'v6' ? 'check' : 'copy'" :size="13" />
          <span>{{ copiedField === 'v6' ? 'Copied' : 'Copy IPv6' }}</span>
        </button>
      </div>
    </section>

    <!-- Responsive Grid for Large Screens & Fullscreen -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
      <!-- Local Network Interfaces (Takes 4 cols on wide screens) -->
      <section class="lg:col-span-4 card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-3.5">
            <div class="flex items-center gap-2">
              <Icon name="network" :size="16" class="text-accent" />
              <h2 class="font-semibold text-[14px]">Local Interfaces</h2>
            </div>
            <span class="text-[11px] text-fg3 font-mono bg-surface2 px-2 py-0.5 rounded-full">
              {{ status?.local_ips?.length ?? 0 }} active
            </span>
          </div>

          <ul v-if="status?.local_ips?.length" class="space-y-1.5">
            <li
              v-for="ip in status.local_ips"
              :key="ip"
              class="flex items-center justify-between p-2 rounded-lg bg-surface2/40 hover:bg-surface2/80 transition-colors group cursor-pointer"
              @click="copy(ip)"
            >
              <span class="num text-[13px] font-medium text-fg truncate select-all">{{ ip }}</span>
              <button
                class="btn-icon p-1 text-fg3 group-hover:text-accent hover:bg-surface transition-all"
                title="Copy IP"
              >
                <Icon name="copy" :size="13" />
              </button>
            </li>
          </ul>
          <p v-else class="text-fg3 text-[13px] py-4 text-center">No active local network interfaces found.</p>
        </div>

        <div class="mt-4 pt-3 border-t border-line/60 flex items-center justify-between text-[11px] text-fg3">
          <span>Loopback excluded</span>
          <span>Click any address to copy</span>
        </div>
      </section>

      <!-- Recent Event Activity (Takes 8 cols on wide screens) -->
      <section class="lg:col-span-8 card p-5 flex flex-col">
        <div class="flex items-center justify-between mb-3.5">
          <div class="flex items-center gap-2">
            <Icon name="history" :size="16" class="text-accent" />
            <h2 class="font-semibold text-[14px]">Recent Activity</h2>
          </div>
          <span class="text-[12px] text-fg3">Latest changes & status checks</span>
        </div>

        <div v-if="loading" class="space-y-2 py-2">
          <div v-for="i in 4" :key="i" class="skeleton h-8"></div>
        </div>

        <div v-else-if="recent.length === 0" class="py-8 text-center text-fg3 text-[13px]">
          No events recorded yet. The first check writes an initial entry.
        </div>

        <ul v-else class="divide-y divide-line/70 -mx-1">
          <li
            v-for="e in recent"
            :key="e.id"
            class="py-2.5 px-2 flex flex-wrap items-center justify-between gap-2.5 hover:bg-surface2/40 rounded-md transition-colors"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span class="chip shrink-0" :class="kindClass(e.kind)">
                {{ kindLabel(e.kind) }}
              </span>
              <span class="num text-[13px] font-medium text-fg truncate">
                <template v-if="e.kind === 'change' && e.prev_public_ip">
                  <span class="text-fg3 font-normal">{{ e.prev_public_ip }}</span>
                  <span class="text-accent mx-1.5">→</span>
                  <span class="font-semibold">{{ e.public_ip }}</span>
                </template>
                <template v-else>{{ e.public_ip ?? e.prev_public_ip ?? '—' }}</template>
              </span>
            </div>

            <div class="flex items-center gap-2 text-fg3 text-[12px] num ml-auto shrink-0">
              <Icon name="clock" :size="12" />
              <span>{{ fmtDateTime(e.ts) }}</span>
            </div>
          </li>
        </ul>
      </section>
    </div>
  </div>
</template>
