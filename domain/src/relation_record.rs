use crate::relation::Relation;

pub trait RelationRecord {
    type AsyncOutput;
    fn add_relation(&self, relation: &Relation) -> Self::AsyncOutput;
    fn get_parents(&self, task_id: i32) -> Self::AsyncOutput;
    fn get_children(&self, task_id: i32) -> Self::AsyncOutput;
    fn get_relatives(&self, task_id: i32) -> Self::AsyncOutput;
    fn delete_relation(&self, relation: &Relation) -> Self::AsyncOutput;
}
