import("./pkg").then(module => {
    module.init_page({
        // element: document.getElementById("#screen"),
        element: "#screen",
        repo: "kawamuray/decaton",
        branch: "bmt-revise",
        file: "tasks_100k_latency_0ms_concurrency_20-benchmark.json",
        query: "performance.throughput",
    });
});
