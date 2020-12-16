use queues::*;
use std::error::Error;


async fn live_download_as_ts_file<C>(fn_update_ts_urls: impl Fn() -> (Vec<String>, Vec<String>), fn_download_ts_file: impl Fn(url: &str) -> Stream) -> Result<(), Box<dyn Error + Send + Sync>> {
    // #EXT-X-VERSION:(.+)(?:.*\s*#)*EXT-X-MEDIA-SEQUENCE:(.+)\s*((?:(?:.|\s)*?#EXTINF:.*live\s+https:\/\/.*.ts+)+)\s+((?:(?:.|\s)*?#EXT-X-TWITCH-PREFETCH:https:\/\/.*.ts+)+)
    // TODO: extract from text_test
    let version: usize = 3; // FIXME: needs to allow for decimals 3, 3.0, 3.1
    let media_sequence: usize = 99;
    let (ts_urls, ts_prefetch_urls) = fn_update_ts_urls();
    //let ts_urls = vec!["https://test.com/".to_string(), "https://test.com/".to_string(), "https://test.com/".to_string(), "https://test.com/".to_string(), "https://test.com/".to_string()];
    //let ts_prefetch_urls = vec!["https://test.com/".to_string(), "https://test.com/".to_string()];


    let mut vod_ts_queue: Queue<isize> = queue![];
    let mut live_ts_queue: Queue<isize> = queue![];
    let mut missed_live_ts_queue: Queue<isize> = queue![];
    // TODO: swap out with fixed sized queue (buffer)
    // TODO: allow of setting active threads downloading files
    // TODO: allow of limiting bandwidth
    let mut ts_number: usize = 100;
    let mut is_reading: bool = true;

    // FIXME: for cache lines, maybe add range to live_ts_queue(?)
    while is_reading {
        // receive next

        // verify if still in order
        if ts_number >= media_sequence && ts_number < media_sequence + ts_urls.len() + ts_prefetch_urls.len() {
            ts_number += 1;
        } else if ts_number == media_sequence + ts_urls.len() + ts_prefetch_urls.len() {
            // refetch m3u8 playlist
        } else {
            // add missed ranged to missed_live_ts_queue
        }
        //add to live queue
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}