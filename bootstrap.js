import("./pkg").then(cloupe => {
    cloupe.create({
        // element: document.getElementById("#screen"),
        on: "#screen",
        repo: "kawamuray/decaton",
        branch: "bmt-revise",
        data_url: "commit-data/",
        components: {
            show_table: false,
            show_range: false,
        },
        data: [
            {
                title: "0ms latency / Throughput",
                file: "tasks_100k_latency_0ms_concurrency_20-benchmark.json",
                query: "performance.throughput",
            },
            {
                title: "10ms latency / Throughput",
                file: "tasks_10k_latency_10ms_concurrency_20-benchmark.json",
                query: "performance.throughput",
            },
            {
                title: "10ms latency / Execution Time",
                file: "tasks_10k_latency_10ms_concurrency_20-benchmark.json",
                query: "performance.executionTime",
            }
        ],
    });
});
