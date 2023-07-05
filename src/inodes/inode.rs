use super::metadata::Metadata;

pub trait INode {
    fn get_meta(&self) -> &Metadata;
    fn touch(&mut self);
}