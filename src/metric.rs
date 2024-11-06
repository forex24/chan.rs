use crate::Bar;

pub trait MetricModel {
    fn update_bar(&mut self, klu: &Bar);
}
