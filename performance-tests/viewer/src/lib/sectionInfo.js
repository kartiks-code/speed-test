export const SECTION_INFO = {
  "Performance Statistics": {
    title: "Performance Statistics",
    intro:
      "These metrics describe how fast and reliably the server handled requests during the load test. They are the primary signals for comparing throughput and latency across stacks.",
    columns: [
      {
        label: "RPS (req/s)",
        explanation:
          "Requests per second — the number of HTTP requests the server completed each second during the test. This is the headline throughput figure. Higher RPS means the server can handle more concurrent users with the same hardware. A low RPS under fixed load indicates CPU saturation, slow I/O, or insufficient concurrency headroom.",
      },
      {
        label: "Error %",
        explanation:
          "The percentage of requests that returned a non-2xx HTTP response or timed out. This should be as close to 0% as possible. Even a 0.1% error rate means one in a thousand users hits a failure. High error rates under load signal that the server is overwhelmed, connection pools are exhausted, or there are unhandled concurrency bugs.",
      },
      {
        label: "p50 (ms)",
        explanation:
          "Median response latency — half of all requests completed faster than this value. It represents the \"typical\" user experience. If p50 is already high, most users are waiting noticeably. Lower is always better. Large differences in p50 between stacks often reflect serialization cost, framework overhead, or database query efficiency.",
      },
      {
        label: "p90 (ms)",
        explanation:
          "90th-percentile latency — 90% of requests finished within this time. A common SLA target because it filters out most noise while still representing the slowest 10% of users. A large gap between p50 and p90 suggests bursty behaviour such as GC pauses, lock contention, or connection pool wait times.",
      },
      {
        label: "p95 (ms)",
        explanation:
          "95th-percentile latency — only 5% of requests were slower. This is a stricter performance bar often used in availability contracts. High p95 values point to periodic spikes: garbage collection stop-the-world pauses, connection pool exhaustion under burst traffic, or OS scheduler jitter on CPU-bound workloads.",
      },
      {
        label: "p99 (ms)",
        explanation:
          "99th-percentile latency — the worst 1% of requests. Even though this affects only one in a hundred users, it matters enormously for interactive applications and retry-heavy microservice chains. High p99 often exposes lock waits, long GC pauses, or resource starvation that p50 completely hides.",
      },
      {
        label: "Startup (ms)",
        explanation:
          "Time in milliseconds from when Docker started the container to when the server responded to its first readiness probe. Measured with 100 ms poll granularity using a host-side curl against the published port — no Docker-container-per-probe overhead. This includes JVM warm-up, dependency injection, database connection-pool initialisation, and any compile-on-start work. Interpreted languages (Node.js, Python, Ruby) and compiled native binaries (Go, Rust) typically start in under a second; JVM stacks (Spring Boot, Helidon, Quarkus, Ktor) usually take several seconds or more. Startup time matters for autoscaling: slow-starting stacks take longer to replace failed instances and add latency during scale-out events.",
      },
    ],
  },

  "Resource Usage Metrics": {
    title: "Resource Usage Metrics",
    intro:
      "These metrics show how much CPU and memory the server container consumed during the test. They reveal the operational cost of running a stack at load and help size containers for production deployments.",
    columns: [
      {
        label: "RAM avg (MB)",
        explanation:
          "Average resident memory used by the server container throughout the benchmark. Lower memory usage means the stack can be deployed on cheaper instances and leaves headroom for traffic spikes. Unusually high averages can indicate memory leaks, large in-process caches, or JVM heap over-provisioning.",
      },
      {
        label: "RAM peak (MB)",
        explanation:
          "Maximum memory observed at any single sample point during the run. Critical for capacity planning: if peak RAM exceeds the container limit, the process will be OOM-killed, causing crashes and service interruptions. A large gap between average and peak indicates bursty allocation patterns, often tied to request spikes or GC heap expansion.",
      },
      {
        label: "CPU avg (%)",
        explanation:
          "Average CPU utilisation as a percentage of the container's allocated CPU quota during the entire benchmark. High average CPU means the stack is compute-bound and will need more cores to scale. Lower CPU at the same RPS indicates more efficient request processing — less time spent on serialization, reflection, or unnecessary work per request.",
      },
      {
        label: "CPU peak (%)",
        explanation:
          "Maximum CPU utilisation recorded at any sample. When peak CPU hits 100%, the container has exhausted its quota, causing requests to queue and latency to spike. Sustained peaks signal that vertical scaling (more CPUs) or workload optimization is needed. Short spikes are normal at startup; sustained peaks during steady-state load are a bottleneck signal.",
      },
    ],
  },

  "DB Metrics": {
    title: "DB Metrics",
    intro:
      "These counters are collected from PostgreSQL's pg_stat_database view before and after the test. They reveal how each server stack interacts with the database — commit frequency, cache efficiency, and row-level read/write volume.",
    columns: [
      {
        label: "PG xact commit",
        explanation:
          "Total transactions committed in PostgreSQL during the run. Tracks database write activity and transaction granularity. Very high counts relative to RPS can indicate overly fine-grained transactions or missing batching, adding per-commit overhead. Comparing this across stacks shows which implementations commit once per request vs. multiple times per CRUD cycle.",
      },
      {
        label: "PG blks read",
        explanation:
          "Disk blocks read from storage (cache misses). Each block read means PostgreSQL could not find the data in shared_buffers and had to go to disk or OS page cache — orders of magnitude slower than an in-memory hit. Ideally this should be near zero for benchmark workloads that fit in the buffer pool. High values suggest cold cache or insufficient shared_buffers.",
      },
      {
        label: "PG blks hit",
        explanation:
          "Blocks served directly from PostgreSQL's shared_buffers in-memory cache. Higher is always better. The buffer cache hit ratio — blks_hit / (blks_hit + blks_read) — should be above 99% for warm workloads. Low hit rates degrade every query and add unpredictable latency. This metric helps distinguish I/O-bound from CPU-bound query patterns.",
      },
      {
        label: "PG tup ins",
        explanation:
          "Total rows inserted during the benchmark run. Measures write volume at the row level. Comparing tup_inserted across stacks verifies that all implementations exercise the database equally. Outliers can reveal stacks that do extra inserts (e.g. audit rows, redundant upserts) or that skip writes entirely due to caching or bugs.",
      },
      {
        label: "PG tup fetch",
        explanation:
          "Rows fetched from the database and returned to the application after any server-side filtering. High fetch counts relative to query count indicate efficient indexed access — the database is reading approximately the rows that are needed. Very high values can also reveal N+1 query patterns where many small fetches replace one efficient join.",
      },
    ],
  },
};
