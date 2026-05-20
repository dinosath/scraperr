import { For, Show, createSignal } from "solid-js";
import { A } from "@solidjs/router";

interface JobRow {
  id: string;
  url: string;
  status: string;
  time_created: string;
  favorite: boolean;
}

interface Props {
  jobs: JobRow[];
  onDelete: (ids: string[]) => void;
  onToggleFavorite: (id: string, current: boolean) => void;
}

export default function JobsTable(props: Props) {
  const [selected, setSelected] = createSignal<Set<string>>(new Set());

  const toggle = (id: string) => {
    const s = new Set(selected());
    if (s.has(id)) s.delete(id);
    else s.add(id);
    setSelected(s);
  };

  const toggleAll = () => {
    if (selected().size === props.jobs.length) {
      setSelected(new Set());
    } else {
      setSelected(new Set(props.jobs.map((j) => j.id)));
    }
  };

  return (
    <div>
      <div style={{ "margin-bottom": "0.5rem" }}>
        <Show when={selected().size > 0}>
          <button onClick={() => { props.onDelete([...selected()]); setSelected(new Set()); }}>
            Delete Selected ({selected().size})
          </button>
        </Show>
      </div>
      <table style={{ width: "100%", "border-collapse": "collapse" }}>
        <thead>
          <tr style={{ "border-bottom": "2px solid #ccc" }}>
            <th style={{ padding: "0.5rem" }}>
              <input type="checkbox" onChange={toggleAll} checked={selected().size === props.jobs.length && props.jobs.length > 0} />
            </th>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>★</th>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>URL</th>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>Status</th>
            <th style={{ padding: "0.5rem", "text-align": "left" }}>Created</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.jobs}>
            {(job) => (
              <tr style={{ "border-bottom": "1px solid #eee" }}>
                <td style={{ padding: "0.5rem" }}>
                  <input type="checkbox" checked={selected().has(job.id)} onChange={() => toggle(job.id)} />
                </td>
                <td style={{ padding: "0.5rem", cursor: "pointer" }} onClick={() => props.onToggleFavorite(job.id, job.favorite)}>
                  {job.favorite ? "★" : "☆"}
                </td>
                <td style={{ padding: "0.5rem" }}>
                  <A href={`/job/${job.id}`} style={{ color: "#4a90d9" }}>{job.url}</A>
                </td>
                <td style={{ padding: "0.5rem" }}>{job.status}</td>
                <td style={{ padding: "0.5rem" }}>{new Date(job.time_created).toLocaleString()}</td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}
