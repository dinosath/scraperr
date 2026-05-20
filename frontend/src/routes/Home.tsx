import { onMount } from "solid-js";
import JobForm from "~/components/JobForm";
import JobsTable from "~/components/JobsTable";
import { jobsStore, loadJobs, submitJob, deleteJobs, updateJobs } from "~/stores/jobs";

export default function Home() {
  onMount(() => loadJobs());

  const handleSubmit = async (data: Parameters<typeof submitJob>[0]) => {
    await submitJob(data);
  };

  const handleToggleFavorite = async (id: string, current: boolean) => {
    await updateJobs([id], "favorite", !current);
  };

  return (
    <div style={{ "max-width": "1200px", margin: "0 auto" }}>
      <h1>Scraperr</h1>

      <section style={{ "margin-bottom": "2rem" }}>
        <h2>Submit a Scrape Job</h2>
        <JobForm onSubmit={handleSubmit} />
      </section>

      <section>
        <h2>Jobs</h2>
        <JobsTable
          jobs={jobsStore.jobs as any}
          onDelete={deleteJobs}
          onToggleFavorite={handleToggleFavorite}
        />
      </section>
    </div>
  );
}
