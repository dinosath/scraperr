import { useParams } from "@solidjs/router";
import { createSignal, onMount, Show } from "solid-js";
import { getJob } from "~/lib/api";

export default function JobDetail() {
  const params = useParams<{ id: string }>();
  const [job, setJob] = createSignal<any>(null);
  const [error, setError] = createSignal("");

  onMount(async () => {
    try {
      const data = await getJob(params.id);
      setJob(data);
    } catch (e: any) {
      setError(e.message);
    }
  });

  return (
    <div style={{ "max-width": "1200px", margin: "0 auto" }}>
      <h1>Job Detail</h1>
      <Show when={error()}>
        <p style={{ color: "red" }}>{error()}</p>
      </Show>
      <Show when={job()}>
        {(j) => (
          <div>
            <dl>
              <dt><strong>ID</strong></dt>
              <dd>{j().id}</dd>
              <dt><strong>URL</strong></dt>
              <dd><a href={j().url} target="_blank" rel="noopener noreferrer">{j().url}</a></dd>
              <dt><strong>Status</strong></dt>
              <dd>{j().status}</dd>
              <dt><strong>Created</strong></dt>
              <dd>{new Date(j().time_created).toLocaleString()}</dd>
              <dt><strong>Agent Mode</strong></dt>
              <dd>{j().agent_mode ? "Yes" : "No"}</dd>
              <dt><strong>Favorite</strong></dt>
              <dd>{j().favorite ? "★" : "☆"}</dd>
            </dl>
            <h2>Elements</h2>
            <pre style={{ background: "#f4f4f4", padding: "1rem", overflow: "auto" }}>
              {JSON.stringify(j().elements, null, 2)}
            </pre>
            <h2>Results</h2>
            <pre style={{ background: "#f4f4f4", padding: "1rem", overflow: "auto" }}>
              {JSON.stringify(j().result, null, 2)}
            </pre>
          </div>
        )}
      </Show>
    </div>
  );
}
