pub trait Widget {
    type Command;
    type Work;

    fn update(&mut self, work: &Self::Work);
    fn show(&mut self, ui: &mut egui::Ui) -> Option<Self::Command>;
}
