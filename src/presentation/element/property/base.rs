use super::Property;
use super::alignment::Alignment;

/// A base set of properties all [`Element`](super::super::Element)s posess.
pub struct BaseProperties {
    pub position: Property<(f64, f64)>,
    pub anchor: Property<Alignment>,
    pub alignment: Property<Alignment>,
    pub z_index: Property<u32>
}

pub trait BasePropertiesProvider {
    #[allow(dead_code)]
    fn get_base_properties(&self) -> &BaseProperties;
}