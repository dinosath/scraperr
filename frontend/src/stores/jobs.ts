import { createSignal } from "solid-js";
import type { Job } from "~/types";
import * as api from "~/lib/api";

const [jobs, setJobs] = createSignal<Job[]>([]);
const [loading, setLoading] = createSignal(false);

export const jobsStore = {
  get jobs() { return jobs(); },
  get loading() { return loading(); },
};

export async function loadJobs() {
  setLoading(true);
  try {
    const data = await api.retrieveJobs();
    setJobs(data as any);
  } finally {
    setLoading(false);
  }
}

export async function submitJob(params: {
  url: string;
  elements: { name: string; selector: string }[];
  job_options: { multi_page_scrape: boolean; custom_headers: Record<string, string> };
  agent_mode: boolean;
  prompt?: string;
}) {
  const result = await api.submitJob(params);
  await loadJobs();
  return result;
}

export async function deleteJobs(ids: string[]) {
  await api.deleteJobs(ids);
  await loadJobs();
}

export async function updateJobs(ids: string[], field: string, value: any) {
  await api.updateJobs({ ids, field, value });
  await loadJobs();
}
