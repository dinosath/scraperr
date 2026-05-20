import { createSignal, onMount, Show } from "solid-js";
import { getAvgElementsPerLink, getAvgJobsPerDay } from "~/lib/api";

export default function Statistics() {
  const [avgElements, setAvgElements] = createSignal<number | null>(null);
  const [avgJobs, setAvgJobs] = createSignal<number | null>(null);
  const [error, setError] = createSignal("");

  onMount(async () => {
    try {
      const [elemRes, jobsRes] = await Promise.all([
        getAvgElementsPerLink(),
        getAvgJobsPerDay(),
      ]);
      setAvgElements(elemRes.avg_elements_per_link);
      setAvgJobs(jobsRes.avg_jobs_per_day);
    } catch (e: any) {
      setError(e.message);
    }
  });

  return (
    <div style={{ "max-width": "600px", margin: "0 auto" }}>
      <h1>Statistics</h1>
      <Show when={error()}>
        <p style={{ color: "red" }}>{error()}</p>
      </Show>
      <div style={{ display: "grid", "grid-template-columns": "1fr 1fr", gap: "1.5rem" }}>
        <div style={{ background: "#f4f4f4", padding: "1.5rem", "border-radius": "8px", "text-align": "center" }}>
          <h3>Avg Elements / Link</h3>
          <p style={{ "font-size": "2rem", "font-weight": "bold" }}>
            {avgElements() !== null ? avgElements()!.toFixed(2) : "—"}
          </p>
        </div>
        <div style={{ background: "#f4f4f4", padding: "1.5rem", "border-radius": "8px", "text-align": "center" }}>
          <h3>Avg Jobs / Day</h3>
          <p style={{ "font-size": "2rem", "font-weight": "bold" }}>
            {avgJobs() !== null ? avgJobs()!.toFixed(2) : "—"}
          </p>
        </div>
      </div>
    </div>
  );
}
