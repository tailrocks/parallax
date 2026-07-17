use super::{Inner, MemoryStore};
use parallax_model::{ErrorEventRow, HistogramRow, MetricExemplarRow, MetricPointRow};

impl MemoryStore {
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let inner = self.lock();
        (
            inner.spans.len(),
            inner.logs.len(),
            inner.metric_points.len() + inner.histograms.len(),
            inner.error_events.len(),
        )
    }

    /// Drop every in-memory telemetry row. Browser-contract reset uses this
    /// between datasets; product composition never reaches it.
    pub fn clear(&self) {
        let mut inner = self.lock();
        *inner = Inner::default();
        self.error_event_read_calls
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn push_metrics(
        &self,
        points: Vec<MetricPointRow>,
        histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
    ) {
        let mut inner = self.lock();
        inner.metric_points.extend(points);
        inner.histograms.extend(histograms);
        inner.metric_exemplars.extend(exemplars);
    }

    pub fn push_error_events(&self, events: Vec<ErrorEventRow>) {
        self.lock().error_events.extend(events);
    }
}
