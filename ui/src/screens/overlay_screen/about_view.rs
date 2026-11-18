use eframe::egui::{self, FontId, RichText, ScrollArea};

use crate::constants::ABOUT_TEXT_SIZE;

pub(crate) fn draw_about_view(ui: &mut egui::Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.label(RichText::new(r#"FloraCat for Beta Reading

“Dead” by JakeIsConcrete (https://jakeisconcrete.bandcamp.com)

"soft-rain-ambient.mp3" Sound Effect by Mikhail on Pixabay (https://pixabay.com/users/soundsforyou-4861230)

"Liebesfreud"
Composed by Fritz Kreisler (1910)
Performed by Rudolf Deman (violin) & Otto Urack (piano)
Recorded: 1922 (Vox 06076)
Source: Internet Archive (https://archive.org/details/deman-liebesfreud)
Status: Public Domain

"Violin Concerto No. 5 in A Major, K. 219: III. Rondeau - Tempo di Menuetto"
Composed by: Wolfgang Amadeus Mozart (1775)
Performed by: Arthur Catterall (violin) & Orchestra conducted by Sir Hamilton Harty
Recorded: April 10, 1924 (English Columbia L 1592–1595)
Source: Internet Archive (https://archive.org/details/l-1592-5-mozart-concerto-in-a-i)
Status: Public Domain

Theme from Dvořák's Symphony No. 9 "From the New World"
Arranged by: Fritz Kreisler
Performed by: Fritz Kreisler (violin) & Carl Lamson (piano)
Recorded: August 28, 1925 (Victor 1122)
Source: Internet Archive / Great 78 Project
Status: Public Domain

FVN Discord for endless motivation

Built using eframe
"#).font(FontId::proportional(ABOUT_TEXT_SIZE)))
    });
}
