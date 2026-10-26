use super::*;

fn release(tag: &str, prerelease: bool, body: &str) -> serde_json::Value {
    serde_json::json!({
        "tag_name": tag, "draft": false, "prerelease": prerelease,
        "body": body, "assets": []
    })
}

fn notes(
    releases: &[serde_json::Value],
    current: (u64, u64),
    target: &str,
    target_body: Option<&str>,
    channel: UpdateChannel,
) -> Result<Option<String>> {
    let listing = serde_json::to_vec(releases).map_err(Error::ReleaseJson)?;
    history_notes(&listing, current, target, target_body, channel)
}

#[test]
fn releases_between_the_installed_and_target_are_grouped_newest_first() -> anyhow::Result<()> {
    let listing = [
        release("v20260920.3", false, "- the target"),
        release("v20260920.1", false, "- skipped a version"),
        release("v20260919.2", false, "- older note"),
        release("v20260918.1", false, "- older still"),
        // Before the installed version, so not part of what this user gets.
        release("v20260901.1", false, "- ancient"),
    ];
    let grouped = notes(
        &listing,
        (20260917, 1),
        "20260920.3",
        Some("- the target"),
        UpdateChannel::Stable,
    )?
    .context("four releases carry notes, so they are grouped")?;
    let headings: Vec<&str> = grouped
        .lines()
        .filter(|line| line.starts_with("## "))
        .collect();
    assert_eq!(
        headings,
        [
            "## 20260920.3",
            "## 20260920.1",
            "## 20260919.2",
            "## 20260918.1"
        ],
        "{grouped}"
    );
    assert!(!grouped.contains("ancient"), "{grouped}");
    Ok(())
}

#[test]
fn a_single_release_is_not_grouped() -> anyhow::Result<()> {
    let listing = [release("v20260920.3", false, "- the target")];
    assert_eq!(
        notes(
            &listing,
            (20260920, 2),
            "20260920.3",
            Some("- the target"),
            UpdateChannel::Stable
        )?,
        None
    );
    Ok(())
}

#[test]
fn releases_without_notes_are_skipped_rather_than_shown_as_empty_headings() -> anyhow::Result<()> {
    let listing = [
        release("v20260920.2", false, "   "),
        release("v20260920.1", false, "- the only note"),
    ];
    let grouped = notes(
        &listing,
        (20260919, 1),
        "20260920.3",
        Some("- the target"),
        UpdateChannel::Stable,
    )?
    .context("two releases carry notes")?;
    assert!(!grouped.contains("20260920.2"), "{grouped}");
    assert!(grouped.contains("## 20260920.1"), "{grouped}");
    Ok(())
}

#[test]
fn a_stable_install_is_not_shown_prerelease_notes() -> anyhow::Result<()> {
    let listing = [
        release("v20260920.2", true, "- only in a beta"),
        release("v20260920.1", false, "- stable note"),
    ];
    let stable = notes(
        &listing,
        (20260919, 1),
        "20260920.3",
        Some("- the target"),
        UpdateChannel::Stable,
    )?
    .context("two stable releases carry notes")?;
    assert!(!stable.contains("only in a beta"), "{stable}");
    assert!(stable.contains("stable note"), "{stable}");

    // The same release is part of what a beta installation was offered.
    let beta = notes(
        &listing,
        (20260919, 1),
        "20260920.3",
        Some("- the target"),
        UpdateChannel::Beta,
    )?
    .context("two releases carry notes")?;
    assert!(beta.contains("only in a beta"), "{beta}");
    Ok(())
}

#[test]
fn drafts_and_foreign_tags_never_reach_the_grouped_notes() -> anyhow::Result<()> {
    let mut draft = release("v20260920.2", false, "- unpublished");
    draft["draft"] = true.into();
    let listing = [
        draft,
        release("nightly", false, "- not a version"),
        release("v20260920.1-rc.1", false, "- not a calendar version"),
        release("v20260920.1", false, "- published"),
    ];
    let grouped = notes(
        &listing,
        (20260919, 1),
        "20260920.3",
        Some("- the target"),
        UpdateChannel::Stable,
    )?
    .context("two published calendar releases carry notes")?;
    assert!(!grouped.contains("unpublished"), "{grouped}");
    assert!(!grouped.contains("not a version"), "{grouped}");
    assert!(!grouped.contains("rc.1"), "{grouped}");
    Ok(())
}

#[test]
fn the_grouped_list_is_capped() -> anyhow::Result<()> {
    let listing: Vec<serde_json::Value> = (1..=40)
        .map(|counter| release(&format!("v20260920.{counter}"), false, "- note"))
        .collect();
    let grouped = notes(
        &listing,
        (20260901, 1),
        "20260921.1",
        Some("- the target"),
        UpdateChannel::Stable,
    )?
    .context("many releases carry notes")?;
    let headings = grouped
        .lines()
        .filter(|line| line.starts_with("## "))
        .count();
    assert_eq!(headings, MAX_GROUPED_RELEASES, "{grouped}");
    // The cap keeps the newest, since those are the ones being installed.
    assert!(grouped.starts_with("## 20260921.1"), "{grouped}");
    Ok(())
}
