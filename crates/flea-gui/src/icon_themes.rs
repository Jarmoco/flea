//! Bounded, asynchronous raster cache for icons requested by a visible view.
use crate::g2::pv1;
use gpui::{App, Bounds, Global, Pixels, RenderImage, Window};
use std::{
    collections::{HashMap, VecDeque},
    sync::{atomic::AtomicU64, Arc},
};

const MAX_ICONS: usize = 512;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Key {
    theme: String,
    name: String,
    logical: u32,
    scale: u32,
}

#[derive(Default)]
pub struct IconCache {
    entries: HashMap<Key, Option<Arc<RenderImage>>>,
    order: VecDeque<Key>,
}
impl Global for IconCache {}

/// Ask for a named icon. Call only for rows actually exposed by a viewport.
/// The built-in mark remains the fallback until a raster is ready or when lookup fails.
pub fn request(theme: &str, name: &str, logical: u32, window: &Window, cx: &mut App) {
    if theme.is_empty() || name.is_empty() || logical == 0 {
        return;
    }
    let scale = window.scale_factor().ceil().max(1.) as u32;
    let key = Key {
        theme: theme.into(),
        name: name.into(),
        logical,
        scale,
    };
    let cache = cx.default_global::<IconCache>();
    if cache.entries.contains_key(&key) {
        return;
    }
    while cache.entries.len() >= MAX_ICONS {
        let Some(old) = cache.order.pop_front() else {
            break;
        };
        cache.entries.remove(&old);
    }
    cache.order.push_back(key.clone());
    cache.entries.insert(key.clone(), None);
    let renderer = cx.svg_renderer();
    let work_key = key.clone();
    let work = cx.background_executor().spawn(async move {
        let roots = flea::icon_packs::session_roots();
        let path = flea::icon_packs::resolver(&work_key.theme, roots).resolve(
            &work_key.name,
            work_key.logical,
            work_key.scale,
        )?;
        let side = work_key.logical.saturating_mul(work_key.scale);
        Some(pv1::raster_run(&pv1::RasterJob {
            raster: pv1::raster_for_path(&path, renderer),
            path,
            size: (side, side),
            epoch: 0,
            latest: Arc::new(AtomicU64::new(0)),
        }))
    });
    cx.spawn(async move |cx| {
        let Some(pv1::RasterOutcome::Raster(image)) = work.await else {
            return;
        };
        cx.update(|cx| {
            let cache = cx.default_global::<IconCache>();
            if let Some(slot) = cache.entries.get_mut(&key) {
                *slot = Some(image);
                cx.refresh_windows();
            }
        });
    })
    .detach();
}

/// Paint a previously requested icon, or return false so the caller draws its fallback.
pub fn paint(
    theme: &str,
    name: &str,
    logical: u32,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let key = Key {
        theme: theme.into(),
        name: name.into(),
        logical,
        scale: window.scale_factor().ceil().max(1.) as u32,
    };
    let Some(image) = cx
        .try_global::<IconCache>()
        .and_then(|cache| cache.entries.get(&key))
        .and_then(Clone::clone)
    else {
        return false;
    };
    window
        .paint_image(bounds, bounds, Default::default(), image, 0, false)
        .is_ok()
}
