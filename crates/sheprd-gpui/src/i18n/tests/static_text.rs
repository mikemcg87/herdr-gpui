use super::*;

/// A message that takes no arguments is formatted once per catalog, so the
/// text kept has to go when the catalog does. Without that, switching language
/// leaves the window showing the previous one's words until restart.
#[test]
fn switching_locale_drops_text_kept_for_the_previous_catalog() {
    show(&Locale::FALLBACK);
    let english = fl!("sidebar-status-working");
    // Asked again, it comes from the kept copy rather than the catalog.
    assert_eq!(fl!("sidebar-status-working"), english);

    show(&Pseudo::Accented.locale());
    let translated = fl!("sidebar-status-working");
    assert_ne!(translated, english, "kept text outlived its catalog");
    assert_eq!(fl!("sidebar-status-working"), translated);

    show(&Locale::FALLBACK);
    assert_eq!(fl!("sidebar-status-working"), english, "switched back");
}

/// The kept copy is what answers a repeat. Without it this fails, where the
/// locale-switch assertions would still pass on an uncached `text`.
#[test]
fn a_repeated_message_reaches_the_catalog_once() {
    show(&Locale::FALLBACK);
    let before = formatted();
    let first = fl!("sidebar-status-idle");
    assert_eq!(formatted() - before, 1, "the first ask should format once");
    for _ in 0..5 {
        assert_eq!(fl!("sidebar-status-idle"), first);
    }
    assert_eq!(formatted() - before, 1, "a repeat went back to the catalog");
}
