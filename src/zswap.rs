use std::{error::Error, fs};

use crate::debug_permissions::DEBUG_DIR;

/*
same_filled_pages
stored_pages
pool_total_size
duplicate_entry
written_back_pages
reject_compress_poor
reject_kmemcache_fail
reject_alloc_fail
reject_reclaim_fail
pool_limit_hit
*/

#[derive(Debug)]
pub struct ZswapStats {
    pub same_filled_pages: u64,
    pub stored_pages: u64,
    pub pool_total_size: u64,
    pub duplicate_entry: u64,
    pub written_back_pages: u64,
    pub reject_compress_poor: u64,
    pub reject_kmemcache_fail: u64,
    pub reject_alloc_fail: u64,
    pub reject_reclaim_fail: u64,
    pub pool_limit_hit: u64,
}

impl ZswapStats {
    fn new() -> Self {
        Self {
            same_filled_pages: 0,
            stored_pages: 0,
            pool_total_size: 0,
            duplicate_entry: 0,
            written_back_pages: 0,
            reject_compress_poor: 0,
            reject_kmemcache_fail: 0,
            reject_alloc_fail: 0,
            reject_reclaim_fail: 0,
            pool_limit_hit: 0,
        }
    }
}
pub fn read_zswap_stats() -> Result<ZswapStats, &'static str> {
    let mut stats = ZswapStats::new();
    stats.same_filled_pages = get_zswap_stat("same_filled_pages");
    stats.stored_pages = get_zswap_stat("stored_pages");
    stats.pool_total_size = get_zswap_stat("pool_total_size");
    stats.duplicate_entry = get_zswap_stat("duplicate_entry");
    stats.written_back_pages = get_zswap_stat("written_back_pages");
    stats.reject_compress_poor = get_zswap_stat("reject_compress_poor");
    stats.reject_kmemcache_fail = get_zswap_stat("reject_kmemcache_fail");
    stats.reject_alloc_fail = get_zswap_stat("reject_alloc_fail");
    stats.reject_reclaim_fail = get_zswap_stat("reject_reclaim_fail");
    stats.pool_limit_hit = get_zswap_stat("pool_limit_hit");
    Ok(stats)
}

fn get_zswap_stat(name: &str) ->u64{
        fs::read_to_string(format!("{}{}{}", DEBUG_DIR, "/zswap/", name))
            .expect("0").replace('\n', "")
            .parse()
            .unwrap_or(0)
}
