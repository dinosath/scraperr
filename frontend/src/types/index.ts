export interface Job {
  id: string;
  url: string;
  elements: Element[];
  status: string;
  result: Record<string, string>[];
  time_created: string;
  agent_mode: boolean;
  prompt?: string;
  favorite: boolean;
  job_options: JobOptions;
  chat_id?: string;
  user_email?: string;
}

export interface Element {
  name: string;
  selector: string;
}

export interface JobOptions {
  multi_page_scrape: boolean;
  custom_headers: Record<string, string>;
}

export interface CronJob {
  id: string;
  job_id: string;
  cron_expression: string;
  enabled: boolean;
  last_run?: string;
}

export interface User {
  email: string;
  full_name?: string;
}

export interface AuthConfig {
  registration_enabled: boolean;
  recordings_enabled: boolean;
}

export interface StatsAvgElementsPerLink {
  avg_elements_per_link: number;
}

export interface StatsAvgJobsPerDay {
  avg_jobs_per_day: number;
}
