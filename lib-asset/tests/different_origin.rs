use lib_asset::{AssetRoot, FsResolver};

#[test]
fn test_origin() {
    let mut resolver = FsResolver::new();
    resolver.set_root(AssetRoot::Assets, "../assets");
    let quaver_path = resolver.get_path(AssetRoot::Assets, "quaver.ttf");
    assert!(std::fs::exists(quaver_path).unwrap());
}
