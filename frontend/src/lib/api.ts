import { getAccessToken } from "./auth";

const BASE_URL = import.meta.env.VITE_API_URL || "";

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = await getAccessToken();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers,
  });

  if (!res.ok) {
    const text = await res.text().catch(() => "Unknown error");
    throw new Error(`${res.status}: ${text}`);
  }

  return res.json();
}

// Auth
export const checkAuth = () => request<{ registration_enabled: boolean; recordings_enabled: boolean }>("/api/auth/check");
export const getMe = () => request<{ email: string; full_name?: string }>("/api/auth/me");

// Jobs
export const submitJob = (body: {
  url: string;
  elements: { name: string; selector: string }[];
  job_options: { multi_page_scrape: boolean; custom_headers: Record<string, string> };
  agent_mode: boolean;
  prompt?: string;
}) => request<{ id: string; message: string }>("/api/submit-scrape-job", { method: "POST", body: JSON.stringify(body) });

export const retrieveJobs = () =>
  request<{ id: string; url: string; status: string; time_created: string; favorite: boolean }[]>(
    "/api/retrieve-scrape-jobs",
    { method: "POST", body: JSON.stringify({}) }
  );

export const getJob = (id: string) => request<any>(`/api/job/${id}`);

export const updateJobs = (body: { ids: string[]; field: string; value: any }) =>
  request<{ message: string }>("/api/update", { method: "POST", body: JSON.stringify(body) });

export const deleteJobs = (ids: string[]) =>
  request<{ message: string }>("/api/delete-scrape-jobs", { method: "POST", body: JSON.stringify({ ids }) });

export const downloadJobs = (body: { ids: string[]; format: string }) =>
  request<Blob>("/api/download", { method: "POST", body: JSON.stringify(body) });

// Cron
export const scheduleCronJob = (body: { job_id: string; cron_expression: string }) =>
  request<{ id: string }>("/api/schedule-cron-job", { method: "POST", body: JSON.stringify(body) });

export const retrieveCronJobs = () => request<any[]>("/api/retrieve-cron-jobs");

export const deleteCronJob = (id: string) =>
  request<{ message: string }>("/api/delete-cron-job", { method: "POST", body: JSON.stringify({ id }) });

// AI
export const aiChat = (body: { message: string; chat_id?: string }) =>
  request<{ response: string }>("/api/ai", { method: "POST", body: JSON.stringify(body) });

export const checkAi = () => request<{ enabled: boolean }>("/api/ai/check");

// Stats
export const getAvgElementsPerLink = () =>
  request<{ avg_elements_per_link: number }>("/api/statistics/get-average-element-per-link");

export const getAvgJobsPerDay = () =>
  request<{ avg_jobs_per_day: number }>("/api/statistics/get-average-jobs-per-day");
