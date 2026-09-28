<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { Bar } from "vue-chartjs";
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale, Tooltip, Legend, type ChartOptions } from "chart.js";
import PageHeader from "./ui/PageHeader.vue";
import EmptyState from "./ui/EmptyState.vue";
import { api, type Stats } from "../lib/api";
import { fmtDuration, fmtDate } from "../lib/format";

ChartJS.register(BarElement, CategoryScale, LinearScale, Tooltip, Legend);

const props = defineProps<{ eventSeq: number }>();

const days = ref(30);
const stats = ref<Stats | null>(null);
const ranges = [7, 30, 90, 365];

async function load() { stats.value = await api.stats(days.value); }
watch(days, load);
watch(() => props.eventSeq, load);
onMounted(load);

function cssVar(name: string) {
  return `rgb(${getComputedStyle(document.documentElement).getPropertyValue(name).trim()})`;
}

const chartData = computed(() => ({
  labels: stats.value?.per_day.map((d) => d.day.slice(5)) ?? [],
  datasets: [
    { label: "Address changes", data: stats.value?.per_day.map((d) => d.changes) ?? [], backgroundColor: cssVar("--c-accent"), borderRadius: 3, maxBarThickness: 18 },
    { label: "Offline events", data: stats.value?.per_day.map((d) => d.offline) ?? [], backgroundColor: cssVar("--c-danger"), borderRadius: 3, maxBarThickness: 18 },
  ],
}));

const chartOptions = computed<ChartOptions<"bar">>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: { legend: { position: "bottom", labels: { color: cssVar("--c-text2"), boxWidth: 10, boxHeight: 10 } } },
  scales: {
    x: { stacked: true, grid: { display: false }, ticks: { color: cssVar("--c-text3"), maxTicksLimit: 12 } },
    y: { stacked: true, beginAtZero: true, grid: { color: cssVar("--c-border") }, ticks: { color: cssVar("--c-text3"), precision: 0 } },
  },
}));

const totalStay = computed(() => stats.value?.stays.reduce((a, s) => a + s.seconds, 0) ?? 0);
const hasData = computed(() => (stats.value?.stays.length ?? 0) > 0 || (stats.value?.offline_events ?? 0) > 0);
</script>

<template>
  <PageHeader title="Reports" subtitle="How stable your connection and address have been.">
    <div class="inline-flex rounded-md border border-line overflow-hidden">
      <button
        v-for="r in ranges" :key="r"
        class="px-3 py-1.5 text-[13px] transition-colors"
        :class="days === r ? 'bg-surface2 text-fg font-medium' : 'text-fg2 hover:bg-surface2/60'"
        @click="days = r"
      >{{ r }}d</button>
    </div>
  </PageHeader>

  <div v-if="!stats" class="grid grid-cols-4 gap-4"><div v-for="i in 4" :key="i" class="skeleton h-20"></div></div>

  <EmptyState v-else-if="!hasData" title="Not enough data yet" body="Reports fill in as the tracker logs events. Come back after a few checks." />

  <template v-else>
    <div class="grid grid-cols-2 lg:grid-cols-5 gap-4 mb-5">
      <div class="card p-4">
        <p class="num text-2xl font-semibold">{{ stats.availability_pct.toFixed(stats.availability_pct >= 99.95 ? 0 : 1) }}%</p>
        <p class="text-fg2 text-[13px]">Online time</p>
      </div>
      <div class="card p-4">
        <p class="num text-2xl font-semibold">{{ stats.total_changes }}</p>
        <p class="text-fg2 text-[13px]">Address changes</p>
      </div>
      <div class="card p-4">
        <p class="num text-2xl font-semibold">{{ stats.unique_ips }}</p>
        <p class="text-fg2 text-[13px]">Unique addresses</p>
      </div>
      <div class="card p-4">
        <p class="num text-2xl font-semibold">{{ fmtDuration(stats.avg_seconds_between_changes) }}</p>
        <p class="text-fg2 text-[13px]">Average time per address</p>
      </div>
      <div class="card p-4">
        <p class="num text-2xl font-semibold">{{ stats.offline_events }}</p>
        <p class="text-fg2 text-[13px]">Offline events</p>
      </div>
    </div>

    <section class="card p-5 mb-5">
      <h2 class="font-medium mb-4">Events per day</h2>
      <div class="h-56"><Bar :data="chartData" :options="chartOptions" /></div>
    </section>

    <section class="card overflow-hidden mb-5">
      <h2 class="font-medium px-5 pt-5 pb-3">Time on each address</h2>
      <div class="overflow-x-auto">
        <table class="data">
          <thead>
            <tr>
              <th>Address</th>
              <th>Label</th>
              <th>Provider</th>
              <th class="w-full">Share</th>
              <th class="text-right">Time</th>
              <th class="text-right">Assigned</th>
              <th>First seen</th>
              <th>Last seen</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in stats.stays" :key="s.public_ip">
              <td class="num selectable whitespace-nowrap">{{ s.public_ip }}</td>
              <td><span v-if="s.label" class="chip bg-accent/15 text-accent">{{ s.label }}</span><span v-else class="text-fg3">—</span></td>
              <td class="text-fg2 whitespace-nowrap">{{ s.isp ?? '—' }}</td>
              <td>
                <div class="h-2 rounded bg-surface2 overflow-hidden min-w-32">
                  <div class="h-full bg-accent" :style="{ width: (totalStay ? (s.seconds / totalStay) * 100 : 0) + '%' }"></div>
                </div>
              </td>
              <td class="num text-right whitespace-nowrap">{{ fmtDuration(s.seconds) }}</td>
              <td class="num text-right text-fg2">{{ s.count }}×</td>
              <td class="num text-fg2 whitespace-nowrap">{{ fmtDate(s.first_seen) }}</td>
              <td class="num text-fg2 whitespace-nowrap">{{ fmtDate(s.last_seen) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section v-if="stats.isps.length" class="card p-5 max-w-lg">
      <h2 class="font-medium mb-3">Providers seen</h2>
      <ul class="divide-y divide-line">
        <li v-for="i in stats.isps" :key="i.isp" class="py-2 flex justify-between gap-4">
          <span>{{ i.isp }}</span>
          <span class="num text-fg2">{{ i.count }} event{{ i.count === 1 ? '' : 's' }}</span>
        </li>
      </ul>
    </section>
  </template>
</template>
