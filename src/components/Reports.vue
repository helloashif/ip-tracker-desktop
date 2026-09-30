<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { Bar } from "vue-chartjs";
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale, Tooltip, Legend, type ChartOptions } from "chart.js";
import PageHeader from "./ui/PageHeader.vue";
import EmptyState from "./ui/EmptyState.vue";
import Icon from "./ui/Icon.vue";
import { api, type Stats } from "../lib/api";
import { fmtDuration } from "../lib/format";

ChartJS.register(BarElement, CategoryScale, LinearScale, Tooltip, Legend);

const props = defineProps<{ eventSeq: number }>();

const days = ref(30);
const stats = ref<Stats | null>(null);
const ranges = [7, 30, 90, 365];

async function load() {
  stats.value = await api.stats(days.value);
}
watch(days, load);
watch(() => props.eventSeq, load);
onMounted(load);

function cssVar(name: string) {
  return `rgb(${getComputedStyle(document.documentElement).getPropertyValue(name).trim()})`;
}

const chartData = computed(() => ({
  labels: stats.value?.per_day.map((d) => d.day.slice(5)) ?? [],
  datasets: [
    {
      label: "Address changes",
      data: stats.value?.per_day.map((d) => d.changes) ?? [],
      backgroundColor: cssVar("--c-accent"),
      borderRadius: 4,
      maxBarThickness: 22,
    },
    {
      label: "Offline events",
      data: stats.value?.per_day.map((d) => d.offline) ?? [],
      backgroundColor: cssVar("--c-danger"),
      borderRadius: 4,
      maxBarThickness: 22,
    },
  ],
}));

const chartOptions = computed<ChartOptions<"bar">>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: "bottom",
      labels: {
        color: cssVar("--c-text2"),
        boxWidth: 12,
        boxHeight: 12,
        font: { size: 12, weight: 500 },
        padding: 16,
      },
    },
    tooltip: {
      backgroundColor: cssVar("--c-surface"),
      titleColor: cssVar("--c-text"),
      bodyColor: cssVar("--c-text2"),
      borderColor: cssVar("--c-border"),
      borderWidth: 1,
      padding: 10,
      boxPadding: 4,
      usePointStyle: true,
    },
  },
  scales: {
    x: {
      stacked: true,
      grid: { display: false },
      ticks: { color: cssVar("--c-text3"), maxTicksLimit: 14, font: { size: 11 } },
    },
    y: {
      stacked: true,
      beginAtZero: true,
      grid: { color: cssVar("--c-border") },
      ticks: { color: cssVar("--c-text3"), precision: 0, font: { size: 11 } },
    },
  },
}));

const totalStay = computed(() => stats.value?.stays.reduce((a, s) => a + s.seconds, 0) ?? 0);
const hasData = computed(() => (stats.value?.stays.length ?? 0) > 0 || (stats.value?.offline_events ?? 0) > 0);
</script>

<template>
  <div class="space-y-6">
    <PageHeader title="Reports" subtitle="Analyze your connection stability, uptime, and IP address turnover.">
      <!-- Native segmented control button group -->
      <div class="inline-flex rounded-lg border border-line bg-surface p-1 gap-1 shadow-xs">
        <button
          v-for="r in ranges"
          :key="r"
          class="px-3 py-1 rounded-md text-[12px] font-medium transition-all"
          :class="days === r ? 'bg-accent text-accent-fg shadow-xs font-semibold' : 'text-fg2 hover:text-fg hover:bg-surface2'"
          @click="days = r"
        >
          {{ r }} days
        </button>
      </div>
    </PageHeader>

    <div v-if="!stats" class="grid grid-cols-2 lg:grid-cols-5 gap-4">
      <div v-for="i in 5" :key="i" class="skeleton h-24"></div>
    </div>

    <EmptyState
      v-else-if="!hasData"
      title="Not enough telemetry data"
      body="Reports populate as IP Tracker observes changes over time. Check back after several hours of tracking."
    />

    <template v-else>
      <!-- Telemetry Metric KPI Cards -->
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4">
        <div class="card p-4 flex flex-col justify-between">
          <div class="flex items-center justify-between text-fg3 mb-2">
            <span class="text-[12px] font-medium">Uptime / Availability</span>
            <span class="h-2 w-2 rounded-full bg-ok"></span>
          </div>
          <p class="num text-2xl sm:text-3xl font-bold tracking-tight text-fg">
            {{ stats.availability_pct.toFixed(stats.availability_pct >= 99.95 ? 0 : 1) }}%
          </p>
          <span class="text-fg3 text-[11px] mt-1">Connection availability</span>
        </div>

        <div class="card p-4 flex flex-col justify-between">
          <div class="flex items-center justify-between text-fg3 mb-2">
            <span class="text-[12px] font-medium">Address Changes</span>
            <Icon name="network" :size="14" class="text-accent" />
          </div>
          <p class="num text-2xl sm:text-3xl font-bold tracking-tight text-fg">
            {{ stats.total_changes }}
          </p>
          <span class="text-fg3 text-[11px] mt-1">Total IP turnover events</span>
        </div>

        <div class="card p-4 flex flex-col justify-between">
          <div class="flex items-center justify-between text-fg3 mb-2">
            <span class="text-[12px] font-medium">Unique Addresses</span>
            <Icon name="globe" :size="14" class="text-fg2" />
          </div>
          <p class="num text-2xl sm:text-3xl font-bold tracking-tight text-fg">
            {{ stats.unique_ips }}
          </p>
          <span class="text-fg3 text-[11px] mt-1">Different IPs assigned</span>
        </div>

        <div class="card p-4 flex flex-col justify-between">
          <div class="flex items-center justify-between text-fg3 mb-2">
            <span class="text-[12px] font-medium">Avg Time / IP</span>
            <Icon name="clock" :size="14" class="text-fg2" />
          </div>
          <p class="num text-xl sm:text-2xl font-bold tracking-tight text-fg truncate">
            {{ fmtDuration(stats.avg_seconds_between_changes) }}
          </p>
          <span class="text-fg3 text-[11px] mt-1">Average retention duration</span>
        </div>

        <div class="card p-4 flex flex-col justify-between">
          <div class="flex items-center justify-between text-fg3 mb-2">
            <span class="text-[12px] font-medium">Offline Events</span>
            <span class="h-2 w-2 rounded-full" :class="stats.offline_events > 0 ? 'bg-danger' : 'bg-fg3'"></span>
          </div>
          <p class="num text-2xl sm:text-3xl font-bold tracking-tight text-fg">
            {{ stats.offline_events }}
          </p>
          <span class="text-fg3 text-[11px] mt-1">Connection loss detected</span>
        </div>
      </div>

      <!-- Chart Section (Dynamically scales on Fullscreen) -->
      <section class="card p-5 sm:p-6 shadow-sm">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <Icon name="reports" :size="16" class="text-accent" />
            <h2 class="font-semibold text-[14px]">Daily Activity Distribution</h2>
          </div>
          <span class="text-[12px] text-fg3">Daily address changes and disconnects</span>
        </div>

        <!-- Dynamic Responsive Height for Fullscreen Scaling -->
        <div class="h-64 sm:h-72 lg:h-80 2xl:h-96 w-full">
          <Bar :data="chartData" :options="chartOptions" />
        </div>
      </section>

      <!-- Bottom Layout: Table & Providers -->
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
        <!-- Address Stay Table (8 cols on wide screens) -->
        <section class="lg:col-span-8 card overflow-hidden flex flex-col justify-between">
          <div>
            <div class="p-5 pb-3">
              <h2 class="font-semibold text-[14px]">Time on Each Address</h2>
              <p class="text-fg3 text-[12px] mt-0.5">Duration and turnover breakdown for each observed address</p>
            </div>

            <div class="overflow-x-auto w-full">
              <table class="data">
                <thead>
                  <tr>
                    <th>Address</th>
                    <th>Label</th>
                    <th>Provider</th>
                    <th class="w-1/3">Usage Share</th>
                    <th class="text-right">Duration</th>
                    <th class="text-right">Hits</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="s in stats.stays" :key="s.public_ip">
                    <td class="num font-medium selectable whitespace-nowrap">{{ s.public_ip }}</td>
                    <td>
                      <span v-if="s.label" class="chip bg-accent/15 text-accent border border-accent/25">{{ s.label }}</span>
                      <span v-else class="text-fg3">—</span>
                    </td>
                    <td class="text-fg2 whitespace-nowrap">{{ s.isp ?? '—' }}</td>
                    <td>
                      <div class="flex items-center gap-2">
                        <div class="h-2 flex-1 rounded-full bg-surface2 overflow-hidden">
                          <div
                            class="h-full bg-accent rounded-full transition-all duration-300"
                            :style="{ width: (totalStay ? (s.seconds / totalStay) * 100 : 0) + '%' }"
                          ></div>
                        </div>
                        <span class="num text-[11px] text-fg3 w-10 text-right">
                          {{ totalStay ? Math.round((s.seconds / totalStay) * 100) : 0 }}%
                        </span>
                      </div>
                    </td>
                    <td class="num text-right whitespace-nowrap font-medium">{{ fmtDuration(s.seconds) }}</td>
                    <td class="num text-right text-fg2">{{ s.count }}×</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </section>

        <!-- Providers Breakdown (4 cols on wide screens) -->
        <section v-if="stats.isps.length" class="lg:col-span-4 card p-5 flex flex-col justify-between">
          <div>
            <div class="flex items-center gap-2 mb-3">
              <Icon name="globe" :size="16" class="text-accent" />
              <h2 class="font-semibold text-[14px]">Providers Observed</h2>
            </div>

            <ul class="divide-y divide-line/70 -mx-1">
              <li v-for="i in stats.isps" :key="i.isp" class="py-3 px-1 flex items-center justify-between gap-3">
                <span class="font-medium text-fg text-[13px] truncate">{{ i.isp }}</span>
                <span class="chip bg-surface2 text-fg2 text-[11px] shrink-0 num">
                  {{ i.count }} event{{ i.count === 1 ? '' : 's' }}
                </span>
              </li>
            </ul>
          </div>

          <div class="mt-4 pt-3 border-t border-line/60 text-fg3 text-[11px]">
            <span>Aggregated across selected time window</span>
          </div>
        </section>
      </div>
    </template>
  </div>
</template>
