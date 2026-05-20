import { createSignal, For, onMount, Show } from "solid-js";
import { retrieveCronJobs, scheduleCronJob, deleteCronJob } from "~/lib/api";

export default function CronJobs() {
  const [crons, setCrons] = createSignal<any[]>([]);
  const [jobId, setJobId] = createSignal("");
  const [cronExpr, setCronExpr] = createSignal("");
  const [error, setError] = createSignal("");

  const load = async () => {
    try {
      const data = await retrieveCronJobs();
      setCrons(data);
    } catch (e: any) {
      setError(e.message);
    }
  };

  onMount(load);

  const handleSchedule = async (e: Event) => {
    e.preventDefault();
    setError("");
    try {
      await scheduleCronJob({ job_id: jobId(), cron_expression: cronExpr() });
      setJobId("");
      setCronExpr("");
      await load();
    } catch (err: any) {
      setError(err.message);
    }
  };

  const handleDelete = async (id: string) => {
    await deleteCronJob(id);
    await load();
  };

  return (
    <div style={{ "max-width": "900px", margin: "0 auto" }}>
      <h1>Cron Jobs</h1>
      <Show when={error()}>
        <p style={{ color: "red" }}>{error()}</p>
      </Show>

      <form onSubmit={handleSchedule} style={{ display: "flex", gap: "0.5rem", "margin-bottom": "1rem" }}>
        <input
          placeholder="Job ID"
          value={jobId()}
          onInput={(e) => setJobId(e.currentTarget.value)}
          required
          style={{ flex: 1, padding: "0.5rem" }}
        />
        <input
          placeholder="Cron Expression (e.g., 0 */6 * * *)"
          value={cronExpr()}
          onInput={(e) => setCronExpr(e.currentTarget.value)}
          required
          style={{ flex: 1, padding: "0.5rem" }}
        />
        <button type="submit">Schedule</button>
      </form>

      <table style={{ width: "100%", "border-collapse": "collapse" }}>
        <thead>
          <tr style={{ "border-bottom": "2px solid #ccc" }}>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>ID</th>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>Job ID</th>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>Schedule</th>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>Enabled</th>
            <th style={{ padding: "0.5rem" }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          <For each={crons()}>
            {(cron) => (
              <tr style={{ "border-bottom": "1px solid #eee" }}>
                <td style={{ padding: "0.5rem" }}>{cron.id}</td>
                <td style={{ padding: "0.5rem" }}>{cron.job_id}</td>
                <td style={{ padding: "0.5rem" }}><code>{cron.cron_expression}</code></td>
                <td style={{ padding: "0.5rem" }}>{cron.enabled ? "Yes" : "No"}</td>
                <td style={{ padding: "0.5rem", "text-align": "center" }}>
                  <button onClick={() => handleDelete(cron.id)}>Delete</button>
                </td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}
