use::mein_core::{Nachricht,Rolle};
fn main() {
    let nachricht = Nachricht::neu(Rolle::Benutzer,"Hallo wie gehts");
    println!("{:?}",nachricht);
}
