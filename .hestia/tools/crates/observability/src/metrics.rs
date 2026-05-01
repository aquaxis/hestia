use prometheus::{
    self, HistogramOpts, HistogramVec, IntCounterVec, IntGaugeVec, Opts, Registry,
};

pub struct Metrics {
    registry: Registry,
    pub build_total: IntCounterVec,
    pub build_duration_seconds: HistogramVec,
    pub agent_active: IntGaugeVec,
    pub agent_pending_tasks: IntGaugeVec,
    pub llm_requests_total: IntCounterVec,
    pub llm_tokens_total: IntCounterVec,
    pub llm_latency_seconds: HistogramVec,
    pub rag_retrieval_seconds: HistogramVec,
    pub rag_hit_ratio: IntGaugeVec,
    pub container_build_total: IntCounterVec,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let build_total = IntCounterVec::new(
            Opts::new("hestia_build_total", "Total number of builds"),
            &["conductor", "status"],
        )?;
        registry.register(Box::new(build_total.clone()))?;

        let build_duration_seconds = HistogramVec::new(
            HistogramOpts::new("hestia_build_duration_seconds", "Build duration in seconds"),
            &["conductor"],
        )?;
        registry.register(Box::new(build_duration_seconds.clone()))?;

        let agent_active = IntGaugeVec::new(
            Opts::new("hestia_agent_active", "Number of active agents"),
            &["conductor"],
        )?;
        registry.register(Box::new(agent_active.clone()))?;

        let agent_pending_tasks = IntGaugeVec::new(
            Opts::new("hestia_agent_pending_tasks", "Number of pending tasks"),
            &["conductor"],
        )?;
        registry.register(Box::new(agent_pending_tasks.clone()))?;

        let llm_requests_total = IntCounterVec::new(
            Opts::new("hestia_llm_requests_total", "Total LLM requests"),
            &["model", "status"],
        )?;
        registry.register(Box::new(llm_requests_total.clone()))?;

        let llm_tokens_total = IntCounterVec::new(
            Opts::new("hestia_llm_tokens_total", "Total LLM tokens used"),
            &["model", "type"],
        )?;
        registry.register(Box::new(llm_tokens_total.clone()))?;

        let llm_latency_seconds = HistogramVec::new(
            HistogramOpts::new("hestia_llm_latency_seconds", "LLM request latency"),
            &["model"],
        )?;
        registry.register(Box::new(llm_latency_seconds.clone()))?;

        let rag_retrieval_seconds = HistogramVec::new(
            HistogramOpts::new("hestia_rag_retrieval_seconds", "RAG retrieval latency"),
            &["conductor"],
        )?;
        registry.register(Box::new(rag_retrieval_seconds.clone()))?;

        let rag_hit_ratio = IntGaugeVec::new(
            Opts::new("hestia_rag_hit_ratio", "RAG cache hit ratio"),
            &["conductor"],
        )?;
        registry.register(Box::new(rag_hit_ratio.clone()))?;

        let container_build_total = IntCounterVec::new(
            Opts::new("hestia_container_build_total", "Total container builds"),
            &["conductor", "status"],
        )?;
        registry.register(Box::new(container_build_total.clone()))?;

        Ok(Self {
            registry,
            build_total,
            build_duration_seconds,
            agent_active,
            agent_pending_tasks,
            llm_requests_total,
            llm_tokens_total,
            llm_latency_seconds,
            rag_retrieval_seconds,
            rag_hit_ratio,
            container_build_total,
        })
    }

    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("failed to create metrics registry")
    }
}