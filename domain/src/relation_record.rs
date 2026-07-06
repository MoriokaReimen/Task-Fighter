use anyhow::Result;

pub trait RelationRecord {
    type AsyncOutput;
    fn get_next_relation_id(&self) -> Result<i32>;
    fn get_parents(&self, task_id: i32) -> Self::AsyncOutput;
    fn get_children(&self, task_id: i32) -> Self::AsyncOutput;
    fn get_relatives(&self, task_id: i32) -> Self::AsyncOutput;
    fn add_parent(&self, task_id: i32, parent_id: i32) -> Self::AsyncOutput;
    fn add_child(&self, task_id: i32, child_id: i32) -> Self::AsyncOutput;
    fn delete_relation(&self, parent_id: i32, child_id: i32) -> Self::AsyncOutput;
}
