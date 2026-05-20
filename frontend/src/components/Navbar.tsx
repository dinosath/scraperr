import { A } from "@solidjs/router";
import { Show } from "solid-js";
import { authStore, login, logout } from "~/stores/auth";

export default function Navbar() {
  return (
    <nav style={{
      display: "flex",
      "align-items": "center",
      "justify-content": "space-between",
      padding: "0.75rem 1.5rem",
      "background-color": "#1a1a2e",
      color: "#fff",
    }}>
      <div style={{ display: "flex", gap: "1.5rem", "align-items": "center" }}>
        <A href="/" style={{ color: "#fff", "text-decoration": "none", "font-weight": "bold", "font-size": "1.25rem" }}>
          Scraperr
        </A>
        <A href="/" style={{ color: "#ccc", "text-decoration": "none" }}>Jobs</A>
        <A href="/cron" style={{ color: "#ccc", "text-decoration": "none" }}>Cron</A>
        <A href="/statistics" style={{ color: "#ccc", "text-decoration": "none" }}>Statistics</A>
      </div>
      <div style={{ display: "flex", gap: "1rem", "align-items": "center" }}>
        <Show
          when={authStore.isAuthenticated}
          fallback={<button onClick={login}>Login</button>}
        >
          <span>{authStore.user?.email}</span>
          <button onClick={logout}>Logout</button>
        </Show>
      </div>
    </nav>
  );
}
