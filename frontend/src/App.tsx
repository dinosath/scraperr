import { Router, Route } from "@solidjs/router";
import { Show, createSignal, onMount } from "solid-js";
import { authStore, initAuth } from "~/stores/auth";
import Home from "~/routes/Home";
import JobDetail from "~/routes/JobDetail";
import CronJobs from "~/routes/CronJobs";
import Statistics from "~/routes/Statistics";
import Layout from "~/components/Layout";

export default function App() {
  const [ready, setReady] = createSignal(false);

  onMount(async () => {
    await initAuth();
    setReady(true);
  });

  return (
    <Show when={ready()} fallback={<div>Loading...</div>}>
      <Router root={Layout}>
        <Route path="/" component={Home} />
        <Route path="/job/:id" component={JobDetail} />
        <Route path="/cron" component={CronJobs} />
        <Route path="/statistics" component={Statistics} />
      </Router>
    </Show>
  );
}
