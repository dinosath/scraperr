import { createSignal, For } from "solid-js";
import type { Element } from "~/types";

interface Props {
  onSubmit: (data: {
    url: string;
    elements: Element[];
    agent_mode: boolean;
    prompt?: string;
    job_options: { multi_page_scrape: boolean; custom_headers: Record<string, string> };
  }) => void;
}

export default function JobForm(props: Props) {
  const [url, setUrl] = createSignal("");
  const [elements, setElements] = createSignal<Element[]>([{ name: "", selector: "" }]);
  const [agentMode, setAgentMode] = createSignal(false);
  const [prompt, setPrompt] = createSignal("");
  const [multiPage, setMultiPage] = createSignal(false);

  const addElement = () => setElements([...elements(), { name: "", selector: "" }]);

  const updateElement = (index: number, field: keyof Element, value: string) => {
    const updated = [...elements()];
    updated[index] = { ...updated[index], [field]: value };
    setElements(updated);
  };

  const removeElement = (index: number) => {
    setElements(elements().filter((_, i) => i !== index));
  };

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    props.onSubmit({
      url: url(),
      elements: elements(),
      agent_mode: agentMode(),
      prompt: agentMode() ? prompt() : undefined,
      job_options: {
        multi_page_scrape: multiPage(),
        custom_headers: {},
      },
    });
  };

  return (
    <form onSubmit={handleSubmit} style={{ display: "flex", "flex-direction": "column", gap: "1rem" }}>
      <div>
        <label>URL</label>
        <input
          type="url"
          value={url()}
          onInput={(e) => setUrl(e.currentTarget.value)}
          placeholder="https://example.com"
          required
          style={{ width: "100%", padding: "0.5rem" }}
        />
      </div>

      <div>
        <label>Elements</label>
        <For each={elements()}>
          {(el, i) => (
            <div style={{ display: "flex", gap: "0.5rem", "margin-bottom": "0.5rem" }}>
              <input
                placeholder="Name"
                value={el.name}
                onInput={(e) => updateElement(i(), "name", e.currentTarget.value)}
                style={{ flex: 1, padding: "0.5rem" }}
              />
              <input
                placeholder="CSS Selector"
                value={el.selector}
                onInput={(e) => updateElement(i(), "selector", e.currentTarget.value)}
                style={{ flex: 1, padding: "0.5rem" }}
              />
              <button type="button" onClick={() => removeElement(i())}>✕</button>
            </div>
          )}
        </For>
        <button type="button" onClick={addElement}>+ Add Element</button>
      </div>

      <div style={{ display: "flex", gap: "1rem" }}>
        <label>
          <input type="checkbox" checked={agentMode()} onChange={(e) => setAgentMode(e.currentTarget.checked)} />
          {" "}Agent Mode
        </label>
        <label>
          <input type="checkbox" checked={multiPage()} onChange={(e) => setMultiPage(e.currentTarget.checked)} />
          {" "}Multi-page Scrape
        </label>
      </div>

      {agentMode() && (
        <div>
          <label>Prompt</label>
          <textarea
            value={prompt()}
            onInput={(e) => setPrompt(e.currentTarget.value)}
            placeholder="Describe what to extract..."
            style={{ width: "100%", padding: "0.5rem", "min-height": "80px" }}
          />
        </div>
      )}

      <button type="submit" style={{ padding: "0.75rem", "font-size": "1rem" }}>
        Submit Job
      </button>
    </form>
  );
}
