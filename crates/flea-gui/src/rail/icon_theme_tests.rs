use crate::g2::pl4::rows::RowsModel;

#[test]
fn rail_reads_and_clears_the_shared_icon_choice() {
    let mut model = RowsModel::default();
    let chosen = flea::jsondoc::parse(r#"{"iconTheme":"Example"}"#).unwrap();
    model.adopt_places(&chosen);
    assert_eq!(model.icon_theme, "Example");
    model.adopt_places(&flea::jsondoc::parse("{}").unwrap());
    assert!(model.icon_theme.is_empty());
}

#[test]
fn columns_use_folder_and_file_icon_roles() {
    use crate::columns::{cache::Row, peeks::PeekRow};
    let mut row = PeekRow {
        name: "example".into(),
        dir: true,
        icon: "text-x-generic".into(),
    };
    assert_eq!(Row::Peek(&row).icon_name(), "folder");
    row.dir = false;
    assert_eq!(Row::Peek(&row).icon_name(), "text-x-generic");
}

#[test]
fn grid_defaults_to_builtin_icons() {
    let view = crate::grid::GridView::new(crate::grid::geometry::Metrics::board(), (600., 400.));
    assert!(view.icon_theme.is_empty());
}
