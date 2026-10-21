use gpui_kit::AssetSource;
use mok_gpui::Assets;

#[test]
fn one_source_serves_both_libraries() {
    let kit = gpui_kit::assets::Assets;
    let both = Assets::before(gpui_kit::assets::Assets);
    let mok = Assets.list("").unwrap();
    assert!(
        mok.iter().any(|path| path.starts_with("fonts/")),
        "Mok serves its fonts"
    );
    assert!(
        mok.iter().any(|path| path.starts_with("icons/")),
        "Mok serves the icons its code names"
    );
    for path in &mok {
        let served = both.load(path).unwrap().expect("Mok's file loads");
        assert_eq!(
            served,
            Assets.load(path).unwrap().unwrap(),
            "{path} is Mok's"
        );
    }
    let icons = kit.list("icons/").unwrap();
    assert!(!icons.is_empty(), "Kit lists its icons");
    for path in &icons {
        let served = both.load(path).unwrap().expect("Kit's icon loads");
        if Assets.load(path).unwrap().is_none() {
            assert_eq!(served, kit.load(path).unwrap().unwrap(), "{path} is Kit's");
        }
    }
}
